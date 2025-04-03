use std::collections::{HashMap, VecDeque};
use crate::analyzer::parser::{Module, Statement, Function};

#[derive(Debug, Clone)]
pub enum SecurityEvent {
    CallToTrusted {
        caller: String,
        callee: String,
        context: String,
    },
    ReturnToUntrusted {
        from: String,
        to: String,
        context: String,
    },
    BoundaryCrossing {
        from_module: String,
        to_module: String,
        kind: BoundaryKind,
    },
    StateAccess {
        module: String,
        field: String,
        kind: AccessKind,
    },
}

#[derive(Debug, Clone)]
pub enum BoundaryKind {
    TrustedToUntrusted,
    UntrustedToTrusted,
    CrossModule,
}

#[derive(Debug, Clone)]
pub enum AccessKind {
    Read,
    Write,
    Transfer,
}

#[derive(Debug)]
pub struct CallContext {
    pub caller: String,
    pub callee: String,
    pub boundary_crossed: bool,
    pub stack_depth: usize,
}

#[derive(Debug)]
pub struct TraceAnalyzer {
    events: Vec<SecurityEvent>,
    call_stack: VecDeque<CallContext>,
    current_module: Option<String>,
    trusted_modules: HashMap<String, bool>,
    call_chains: HashMap<String, Vec<String>>,
}

impl TraceAnalyzer {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            call_stack: VecDeque::new(),
            current_module: None,
            trusted_modules: HashMap::new(),
            call_chains: HashMap::new(),
        }
    }

    pub fn analyze_module(&mut self, module: &Module) -> Vec<SecurityEvent> {
        self.current_module = Some(module.name.clone());
        self.events.clear();

        for function in &module.functions {
            self.analyze_function(function);
        }

        self.events.clone()
    }

    fn analyze_function(&mut self, function: &Function) {
        for statement in &function.body {
            self.analyze_statement(statement);
        }
    }

    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Call(name, _) => {
                self.track_call(name);
            }
            Statement::Return(_) => {
                self.track_return();
            }
            Statement::BorrowGlobal(type_name) => {
                self.track_state_access(type_name, AccessKind::Read);
            }
            _ => {}
        }
    }

    fn track_call(&mut self, name: &str) {
        let current_module = self.current_module.clone().unwrap_or_default();
        
        if let Some(called_module) = name.split("::").next() {
            // Track boundary crossing
            if called_module != "Self" {
                let boundary_kind = self.determine_boundary_kind(&current_module, called_module);
                self.events.push(SecurityEvent::BoundaryCrossing {
                    from_module: current_module.clone(),
                    to_module: called_module.to_string(),
                    kind: boundary_kind,
                });
            }

            // Update call context
            let context = CallContext {
                caller: current_module.clone(),
                callee: called_module.to_string(),
                boundary_crossed: called_module != "Self",
                stack_depth: self.call_stack.len(),
            };
            self.call_stack.push_back(context);

            // Update call chain
            self.call_chains
                .entry(current_module.clone())
                .or_default()
                .push(called_module.to_string());

            // Track security event if crossing trust boundary
            if self.is_crossing_trust_boundary(&current_module, called_module) {
                self.events.push(SecurityEvent::CallToTrusted {
                    caller: current_module,
                    callee: called_module.to_string(),
                    context: format!("Call stack depth: {}", self.call_stack.len()),
                });
            }
        }
    }

    fn track_return(&mut self) {
        if let Some(context) = self.call_stack.pop_back() {
            if context.boundary_crossed {
                self.events.push(SecurityEvent::ReturnToUntrusted {
                    from: context.callee,
                    to: context.caller,
                    context: format!("Call stack depth: {}", self.call_stack.len()),
                });
            }
        }
    }

    fn track_state_access(&mut self, field: &str, kind: AccessKind) {
        if let Some(current_module) = &self.current_module {
            self.events.push(SecurityEvent::StateAccess {
                module: current_module.clone(),
                field: field.to_string(),
                kind,
            });
        }
    }

    fn determine_boundary_kind(&self, from: &str, to: &str) -> BoundaryKind {
        let from_trusted = self.trusted_modules.get(from).copied().unwrap_or(false);
        let to_trusted = self.trusted_modules.get(to).copied().unwrap_or(false);

        match (from_trusted, to_trusted) {
            (true, false) => BoundaryKind::TrustedToUntrusted,
            (false, true) => BoundaryKind::UntrustedToTrusted,
            _ => BoundaryKind::CrossModule,
        }
    }

    fn is_crossing_trust_boundary(&self, from: &str, to: &str) -> bool {
        let from_trusted = self.trusted_modules.get(from).copied().unwrap_or(false);
        let to_trusted = self.trusted_modules.get(to).copied().unwrap_or(false);
        from_trusted != to_trusted
    }

    pub fn mark_trusted_module(&mut self, module_name: String) {
        self.trusted_modules.insert(module_name, true);
    }

    pub fn get_call_chain(&self, module_name: &str) -> Option<&Vec<String>> {
        self.call_chains.get(module_name)
    }

    pub fn get_security_events(&self) -> &[SecurityEvent] {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_crossing_detection() {
        let mut analyzer = TraceAnalyzer::new();
        analyzer.mark_trusted_module("trusted".to_string());
        
        let mut module = Module::new("untrusted".to_string());
        let mut function = Function::new("test".to_string());
        function.add_statement(Statement::Call("trusted::func".to_string(), vec![]));
        module.add_function(function);

        let events = analyzer.analyze_module(&module);
        assert!(events.iter().any(|e| matches!(e, 
            SecurityEvent::BoundaryCrossing { 
                kind: BoundaryKind::UntrustedToTrusted, 
                .. 
            }
        )));
    }

    #[test]
    fn test_call_chain_tracking() {
        let mut analyzer = TraceAnalyzer::new();
        
        let mut module = Module::new("test".to_string());
        let mut function = Function::new("func".to_string());
        function.add_statement(Statement::Call("mod1::func".to_string(), vec![]));
        function.add_statement(Statement::Call("mod2::func".to_string(), vec![]));
        module.add_function(function);

        analyzer.analyze_module(&module);
        
        let chain = analyzer.get_call_chain("test").unwrap();
        assert_eq!(chain.len(), 2);
    }
} 