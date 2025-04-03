use std::collections::{HashMap, HashSet};
use crate::analyzer::types::*;
use crate::analyzer::parser::{Statement, Expression, Function};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbstractValue {
    NonRef,     // Non-reference value
    OkRef,      // Safe reference
    InvRef      // Reference to invariant-relevant state
}

#[derive(Debug, Clone)]
pub struct ReferencePath {
    pub elements: Vec<String>,
    pub current_value: AbstractValue,
}

#[derive(Debug)]
pub struct EscapePoint {
    pub location: Location,
    pub path: Vec<String>,
    pub context: String,
    pub value: AbstractValue,
}

#[derive(Debug, Clone)]
pub enum SuiObjectKind {
    Owned,      // Single-owner object
    Shared,     // Shared object
    Immutable,  // Immutable object
    Dynamic     // Dynamic field
}

#[derive(Debug)]
pub struct ObjectInfo {
    pub kind: SuiObjectKind,
    pub has_uid: bool,
    pub capabilities: HashSet<String>,
}

pub struct EscapeAnalyzer {
    locals: HashMap<String, AbstractValue>,
    stack: Vec<AbstractValue>,
    escape_points: Vec<EscapePoint>,
    invariant_fields: HashSet<String>,
    current_path: Option<ReferencePath>,
    function_name: Option<String>,
    object_info: HashMap<String, ObjectInfo>,
}

impl EscapeAnalyzer {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            stack: Vec::new(),
            escape_points: Vec::new(),
            invariant_fields: HashSet::new(),
            current_path: None,
            function_name: None,
            object_info: HashMap::new(),
        }
    }

    pub fn analyze_function(&mut self, function: &Function) -> Result<(), Vec<String>> {
        self.locals.clear();
        self.stack.clear();
        self.function_name = Some(function.name.clone());
        self.current_path = Some(ReferencePath {
            elements: vec![],
            current_value: AbstractValue::NonRef,
        });

        // Initialize parameters
        for param in &function.parameters {
            if param.is_mutable_reference() {
                self.locals.insert(param.name.clone(), AbstractValue::OkRef);
                self.track_path_element(&param.name);
            } else {
                self.locals.insert(param.name.clone(), AbstractValue::NonRef);
            }
        }

        // Analyze function body
        for statement in &function.body {
            if let Err(e) = self.analyze_statement(statement) {
                return Err(vec![e]);
            }
        }

        Ok(())
    }

    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            // Ξimm-BorrowFld-Relevant
            Statement::BorrowField(field) => {
                if self.is_invariant_field(field) {
                    self.stack.push(AbstractValue::InvRef);
                    self.track_path_element(field);
                    self.check_field_escape(field)?;
                } else {
                    let base_value = self.stack.last().cloned().unwrap_or(AbstractValue::NonRef);
                    self.stack.push(base_value);
                    self.track_path_element(field);
                }
                Ok(())
            }

            // Ξimm-BorrowGlobal
            Statement::BorrowGlobal(type_name) => {
                self.stack.push(AbstractValue::InvRef);
                self.track_path_element(&format!("global<{}>", type_name));
                self.check_global_escape(type_name)
            }

            // Ξimm-BorrowLoc
            Statement::BorrowLocal(var) => {
                self.stack.push(AbstractValue::OkRef);
                self.track_path_element(var);
                Ok(())
            }

            // Handle function calls
            Statement::Call(name, args) => {
                self.track_path_element(&format!("call<{}>", name));
                self.check_call_escape(name, args)
            }

            // Ξimm-Return
            Statement::Return(expr) => {
                self.analyze_expression(expr)?;
                self.check_return_escape()
            }

            _ => Ok(())
        }
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Variable(name) => {
                if let Some(value) = self.locals.get(name) {
                    self.stack.push(value.clone());
                    self.track_path_element(name);
                }
                Ok(())
            }
            Expression::FieldAccess(base, field) => {
                self.analyze_expression(base)?;
                if self.is_invariant_field(field) {
                    self.stack.push(AbstractValue::InvRef);
                    self.track_path_element(field);
                    self.check_field_escape(field)?;
                }
                Ok(())
            }
            _ => Ok(())
        }
    }

    fn track_path_element(&mut self, element: &str) {
        if let Some(path) = &mut self.current_path {
            path.elements.push(element.to_string());
            path.current_value = self.stack.last().cloned().unwrap_or(AbstractValue::NonRef);
        }
    }

    fn check_field_escape(&mut self, field: &str) -> Result<(), String> {
        if self.is_invariant_field(field) {
            self.record_escape(format!("Access to invariant field {}", field))
        } else {
            Ok(())
        }
    }

    fn check_global_escape(&mut self, type_name: &str) -> Result<(), String> {
        self.record_escape(format!("Access to global state {}", type_name))
    }

    fn check_call_escape(&mut self, name: &str, _args: &[Expression]) -> Result<(), String> {
        let mut has_inv_ref = false;
        {
            let stack_values = &self.stack;
            for value in stack_values {
                if *value == AbstractValue::InvRef {
                    has_inv_ref = true;
                    break;
                }
            }
        }
        if has_inv_ref {
            self.record_escape(format!("InvRef passed to function {}", name))?;
        }
        Ok(())
    }

    fn check_return_escape(&mut self) -> Result<(), String> {
        for value in &self.stack {
            if *value == AbstractValue::InvRef {
                return self.record_escape("InvRef leaked through return".to_string());
            }
        }
        Ok(())
    }

    fn record_escape(&mut self, context: String) -> Result<(), String> {
        let path = self.current_path.as_ref().map(|p| p.elements.clone()).unwrap_or_default();
        let value = self.stack.last().cloned().unwrap_or(AbstractValue::NonRef);
        
        self.escape_points.push(EscapePoint {
            location: Location::default(),
            path,
            context: format!("{} in function {}", 
                context,
                self.function_name.as_ref().unwrap_or(&"unknown".to_string())
            ),
            value,
        });
        
        Err("Reference escape detected".to_string())
    }

    fn is_invariant_field(&self, field: &str) -> bool {
        self.invariant_fields.contains(field)
    }

    pub fn add_invariant_field(&mut self, field: String) {
        self.invariant_fields.insert(field);
    }

    pub fn get_escape_points(&self) -> &[EscapePoint] {
        &self.escape_points
    }

    pub fn track_object_creation(&mut self, type_name: &str, kind: SuiObjectKind) {
        let info = ObjectInfo {
            kind,
            has_uid: false, // Will be set during initialization check
            capabilities: HashSet::new(),
        };
        self.object_info.insert(type_name.to_string(), info);
    }

    pub fn verify_transfer_safety(&mut self, object: &str) -> Result<(), String> {
        if let Some(info) = self.object_info.get(object) {
            match info.kind {
                SuiObjectKind::Owned => {
                    // Check for proper transfer::transfer usage
                    if !self.has_ownership_check(object) {
                        return Err("Transfer without ownership verification".to_string());
                    }
                }
                SuiObjectKind::Shared => {
                    // Check for consensus sync
                    if !self.has_consensus_check(object) {
                        return Err("Shared object modification without consensus".to_string());
                    }
                }
                SuiObjectKind::Immutable => {
                    // Immutable objects can't be transferred
                    return Err("Attempting to transfer immutable object".to_string());
                }
                SuiObjectKind::Dynamic => {
                    // Check dynamic field operations
                    if !self.verify_dynamic_field_safety(object) {
                        return Err("Unsafe dynamic field operation".to_string());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn check_capability_usage(&mut self, cap: &str) -> Result<(), String> {
        if let Some(_info) = self.object_info.get(cap) {
            // Verify capability is properly owned
            if !self.has_capability_check(cap) {
                return Err("Capability usage without verification".to_string());
            }
            
            // Check for capability leaks
            if self.can_leak_capability(cap) {
                return Err("Potential capability leak detected".to_string());
            }
        }
        Ok(())
    }

    // Sui-specific helper methods
    fn has_ownership_check(&self, _object: &str) -> bool {
        if let Some(path) = &self.current_path {
            // Check for tx_context::sender() == owner pattern
            let has_sender_check = path.elements.iter()
                .any(|e| e.contains("tx_context::sender"));
            
            // Check for owner field access
            let has_owner_access = path.elements.iter()
                .any(|e| e.contains("owner"));

            // Check for assert! or require! pattern
            let has_assertion = path.elements.iter()
                .any(|e| e.contains("assert") || e.contains("require"));

            return has_sender_check && has_owner_access && has_assertion;
        }
        false
    }

    fn has_consensus_check(&self, _object: &str) -> bool {
        if let Some(path) = &self.current_path {
            // Check for consensus::verify pattern
            let has_consensus_verify = path.elements.iter()
                .any(|e| e.contains("consensus::verify"));
            
            // Check for shared object access pattern
            let has_shared_access = path.elements.iter()
                .any(|e| e.contains("shared") || e.contains("consensus"));

            // Check for proper synchronization
            let has_sync = path.elements.iter()
                .any(|e| e.contains("sync") || e.contains("lock"));

            return has_consensus_verify && has_shared_access && has_sync;
        }
        false
    }

    fn verify_dynamic_field_safety(&self, _object: &str) -> bool {
        if let Some(path) = &self.current_path {
            // Check for dynamic field existence check
            let has_existence_check = path.elements.iter()
                .any(|e| e.contains("exists_"));
            
            // Check for proper field access pattern
            let has_field_access = path.elements.iter()
                .any(|e| e.contains("dynamic_field::"));

            // Check for cleanup on removal
            let has_cleanup = path.elements.iter()
                .any(|e| e.contains("remove") && e.contains("cleanup"));

            return has_existence_check && has_field_access && 
                   (path.elements.iter().any(|e| e.contains("add")) || has_cleanup);
        }
        false
    }

    fn has_capability_check(&self, _cap: &str) -> bool {
        if let Some(path) = &self.current_path {
            // Check for capability verification pattern
            let has_cap_check = path.elements.iter()
                .any(|e| e.contains("verify_capability"));
            
            // Check for ownership verification
            let has_owner_check = path.elements.iter()
                .any(|e| e.contains("owner") || e.contains("authorized"));

            // Check for proper capability type
            let has_cap_type = path.elements.iter()
                .any(|e| e.contains("_cap") || e.contains("capability"));

            has_cap_check && has_owner_check && has_cap_type
        } else {
            false
        }
    }

    fn can_leak_capability(&self, _cap: &str) -> bool {
        if let Some(path) = &self.current_path {
            // Check for capability transfer
            let has_transfer = path.elements.iter()
                .any(|e| e.contains("transfer") || e.contains("send"));
            
            // Check for capability storage
            let has_storage = path.elements.iter()
                .any(|e| e.contains("store") || e.contains("save"));

            // Check for capability reference leak
            let has_ref_leak = path.elements.iter()
                .any(|e| e.contains("borrow_mut") || e.contains("&mut"));

            has_transfer || has_storage || has_ref_leak
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn track_sui_specific_patterns(&mut self, statement: &Statement) {
        match statement {
            Statement::Call(name, args) => {
                // Track transfer patterns
                if name.contains("transfer::transfer") {
                    self.track_transfer_pattern(name, args);
                }
                
                // Track dynamic field operations
                if name.contains("dynamic_field") {
                    self.track_dynamic_field_pattern(name, args);
                }
                
                // Track consensus operations
                if name.contains("consensus") {
                    self.track_consensus_pattern(name, args);
                }
            }
            _ => {}
        }
    }

    #[allow(dead_code)]
    fn track_transfer_pattern(&mut self, name: &str, _args: &[Expression]) {
        if let Some(info) = self.object_info.get(name) {
            match info.kind {
                SuiObjectKind::Owned => {
                    // Track ownership transfer
                    self.track_path_element("transfer_owned");
                }
                SuiObjectKind::Shared => {
                    // Track shared object transfer
                    self.track_path_element("transfer_shared");
                }
                _ => {}
            }
        }
    }

    #[allow(dead_code)]
    fn track_dynamic_field_pattern(&mut self, name: &str, _args: &[Expression]) {
        if name.contains("add") {
            self.track_path_element("dynamic_field_add");
        } else if name.contains("remove") {
            self.track_path_element("dynamic_field_remove");
        } else if name.contains("borrow") {
            self.track_path_element("dynamic_field_access");
        }
    }

    #[allow(dead_code)]
    fn track_consensus_pattern(&mut self, name: &str, _args: &[Expression]) {
        if name.contains("verify") {
            self.track_path_element("consensus_verify");
        } else if name.contains("sync") {
            self.track_path_element("consensus_sync");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_field_leak() {
        let mut analyzer = EscapeAnalyzer::new();
        analyzer.add_invariant_field("value".to_string());
        
        let mut function = Function::new("test".to_string());
        function.add_statement(Statement::BorrowField("value".to_string()));
        function.add_statement(Statement::Return(Expression::Variable("x".to_string())));
        
        let result = analyzer.analyze_function(&function);
        assert!(result.is_err());
        assert!(!analyzer.escape_points.is_empty());
        
        let escape = &analyzer.escape_points[0];
        assert!(escape.path.contains(&"value".to_string()));
        assert_eq!(escape.value, AbstractValue::InvRef);
    }

    #[test]
    fn test_safe_reference() {
        let mut analyzer = EscapeAnalyzer::new();
        
        let mut function = Function::new("test".to_string());
        function.add_statement(Statement::BorrowLocal("x".to_string()));
        function.add_statement(Statement::Return(Expression::Variable("x".to_string())));
        
        let result = analyzer.analyze_function(&function);
        assert!(result.is_ok());
        assert!(analyzer.escape_points.is_empty());
    }

    #[test]
    fn test_path_tracking() {
        let mut analyzer = EscapeAnalyzer::new();
        analyzer.add_invariant_field("value".to_string());
        
        let mut function = Function::new("test".to_string());
        function.add_statement(Statement::BorrowLocal("x".to_string()));
        function.add_statement(Statement::BorrowField("value".to_string()));
        
        let _ = analyzer.analyze_function(&function);
        
        let escape = &analyzer.escape_points[0];
        assert_eq!(escape.path, vec!["x".to_string(), "value".to_string()]);
    }
}