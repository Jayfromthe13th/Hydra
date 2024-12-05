use super::types::*;
use super::reference_state::ReferenceStateAnalyzer;
use super::reference_path::ReferencePathAnalyzer;
use std::collections::{HashMap, HashSet};

pub struct ReferenceContextAnalyzer {
    state_analyzer: ReferenceStateAnalyzer,
    path_analyzer: ReferencePathAnalyzer,
    contexts: HashMap<String, ReferenceContext>,
    active_scopes: Vec<ScopeContext>,
    current_function: Option<String>,
}

#[derive(Debug, Clone)]
struct ReferenceContext {
    definition: Location,
    current_state: ReferenceState,
    aliases: HashSet<String>,
    access_paths: Vec<AccessPath>,
    constraints: Vec<ReferenceConstraint>,
}

#[derive(Debug, Clone)]
struct ScopeContext {
    level: usize,
    references: HashSet<String>,
    borrows: HashMap<String, BorrowInfo>,
    conditions: Vec<PathCondition>,
}

#[derive(Debug, Clone)]
struct AccessPath {
    path: Vec<AccessStep>,
    location: Location,
    is_mutable: bool,
}

#[derive(Debug, Clone)]
enum AccessStep {
    Field(FieldId),
    Dereference,
    Index(String),
    Method(String),
}

#[derive(Debug, Clone)]
struct BorrowInfo {
    kind: BorrowKind,
    source: String,
    location: Location,
    is_active: bool,
}

impl ReferenceContextAnalyzer {
    pub fn new() -> Self {
        Self {
            state_analyzer: ReferenceStateAnalyzer::new(),
            path_analyzer: ReferencePathAnalyzer::new(),
            contexts: HashMap::new(),
            active_scopes: vec![ScopeContext::new(0)],
            current_function: None,
        }
    }

    pub fn analyze_function(&mut self, function: &Function) -> Vec<ReferenceLeak> {
        let mut leaks = Vec::new();
        
        // Set current function context
        self.current_function = Some(function.name.clone());
        
        // Enter function scope
        self.enter_scope();
        
        // Initialize parameter contexts
        self.initialize_parameters(function);
        
        // Analyze function body with context tracking
        self.analyze_statements(&function.body, &mut leaks);
        
        // Check for context-based leaks
        self.check_context_safety(&mut leaks);
        
        // Exit function scope
        self.exit_scope(&mut leaks);
        
        // Clear function context
        self.current_function = None;
        
        leaks
    }

    fn initialize_parameters(&mut self, function: &Function) {
        for param in &function.parameters {
            let context = ReferenceContext {
                definition: Location::default(), // Would need proper location
                current_state: self.get_initial_state(&param.param_type),
                aliases: HashSet::new(),
                access_paths: Vec::new(),
                constraints: self.get_parameter_constraints(&param.param_type),
            };
            
            self.contexts.insert(param.name.clone(), context);
            
            if let Some(scope) = self.active_scopes.last_mut() {
                scope.references.insert(param.name.clone());
                
                if let Type::MutableReference(_) = param.param_type {
                    scope.borrows.insert(param.name.clone(), BorrowInfo {
                        kind: BorrowKind::MutableWrite,
                        source: "parameter".to_string(),
                        location: Location::default(),
                        is_active: true,
                    });
                }
            }
        }
    }

    fn analyze_statements(&mut self, statements: &[Statement], leaks: &mut Vec<ReferenceLeak>) {
        for statement in statements {
            match statement {
                Statement::Assignment(var, expr) => {
                    self.analyze_assignment(var, expr, leaks);
                }
                Statement::Return(expr) => {
                    self.analyze_return(expr, leaks);
                }
            }
        }
    }

    fn analyze_assignment(&mut self, var: &str, expr: &Expression, leaks: &mut Vec<ReferenceLeak>) {
        let access_path = self.analyze_expression_path(expr);
        let new_state = self.state_analyzer.evaluate_expression(expr);
        
        // Check for reference leaks through assignment
        if let Some(leak) = self.check_assignment_safety(var, &new_state, &access_path) {
            leaks.push(leak);
        }

        // Update context
        if let Some(context) = self.contexts.get_mut(var) {
            context.current_state = new_state;
            context.access_paths.push(access_path);
            
            // Update aliases
            if let Some(alias_source) = self.get_alias_source(expr) {
                context.aliases.insert(alias_source);
            }
        } else {
            // Create new context for variable
            self.contexts.insert(var.to_string(), ReferenceContext {
                definition: Location::default(), // Would need proper location
                current_state: new_state,
                aliases: self.get_expression_aliases(expr),
                access_paths: vec![access_path],
                constraints: Vec::new(),
            });
        }

        // Update scope information
        if let Some(scope) = self.active_scopes.last_mut() {
            scope.references.insert(var.to_string());
        }
    }

    fn analyze_return(&mut self, expr: &Expression, leaks: &mut Vec<ReferenceLeak>) {
        let access_path = self.analyze_expression_path(expr);
        let return_state = self.state_analyzer.evaluate_expression(expr);
        
        // Check for reference leaks through return
        if let Some(leak) = self.check_return_safety(&return_state, &access_path) {
            leaks.push(leak);
        }
    }

    fn analyze_expression_path(&self, expr: &Expression) -> AccessPath {
        let mut path = Vec::new();
        match expr {
            Expression::Variable(name) => {
                // Base variable access
            }
            Expression::FieldAccess(base, field) => {
                // Add base path
                let mut base_path = self.analyze_expression_path(base);
                path.extend(base_path.path);
                
                // Add field access
                path.push(AccessStep::Field(FieldId {
                    module_name: String::new(), // Would need proper context
                    struct_name: String::new(), // Would need proper context
                    field_name: field.clone(),
                }));
            }
        }
        
        AccessPath {
            path,
            location: Location::default(), // Would need proper location
            is_mutable: self.is_mutable_access(expr),
        }
    }

    fn check_assignment_safety(
        &self,
        var: &str,
        new_state: &ReferenceState,
        access_path: &AccessPath,
    ) -> Option<ReferenceLeak> {
        // Check for various safety violations
        if access_path.is_mutable && self.is_protected_path(access_path) {
            Some(ReferenceLeak {
                location: access_path.location.clone(),
                leaked_field: self.get_leaked_field(access_path),
                context: format!("Mutable reference to protected field assigned to {}", var),
                severity: Severity::High,
            })
        } else {
            None
        }
    }

    fn check_return_safety(
        &self,
        return_state: &ReferenceState,
        access_path: &AccessPath,
    ) -> Option<ReferenceLeak> {
        match return_state {
            ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. } => {
                Some(ReferenceLeak {
                    location: access_path.location.clone(),
                    leaked_field: self.get_leaked_field(access_path),
                    context: "Mutable reference escapes through return".to_string(),
                    severity: Severity::Critical,
                })
            }
            _ => None,
        }
    }

    fn enter_scope(&mut self) {
        let new_level = self.active_scopes.last().map_or(0, |s| s.level + 1);
        self.active_scopes.push(ScopeContext::new(new_level));
    }

    fn exit_scope(&mut self, leaks: &mut Vec<ReferenceLeak>) {
        if let Some(scope) = self.active_scopes.pop() {
            // Check for references that escape their scope
            for var in &scope.references {
                if let Some(context) = self.contexts.get(var) {
                    if self.reference_escapes_scope(var, context) {
                        leaks.push(ReferenceLeak {
                            location: context.definition.clone(),
                            leaked_field: self.get_context_field(context),
                            context: format!("Reference {} escapes its scope", var),
                            severity: Severity::High,
                        });
                    }
                }
            }
        }
    }

    // Helper methods
    fn get_initial_state(&self, ty: &Type) -> ReferenceState {
        match ty {
            Type::MutableReference(_) => ReferenceState::Borrowed {
                kind: BorrowKind::MutableWrite,
                source: "parameter".to_string(),
            },
            _ => ReferenceState::Uninitialized,
        }
    }

    fn get_parameter_constraints(&self, ty: &Type) -> Vec<ReferenceConstraint> {
        let mut constraints = Vec::new();
        match ty {
            Type::MutableReference(_) => {
                constraints.push(ReferenceConstraint::MustBeValid);
                constraints.push(ReferenceConstraint::NoAliasing("parameter".to_string()));
            }
            _ => {}
        }
        constraints
    }

    fn is_mutable_access(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Variable(name) => {
                self.contexts.get(name)
                    .map_or(false, |ctx| matches!(ctx.current_state,
                        ReferenceState::Borrowed { kind: BorrowKind::MutableWrite, .. }
                    ))
            }
            Expression::FieldAccess(base, _) => self.is_mutable_access(base),
        }
    }

    fn is_protected_path(&self, path: &AccessPath) -> bool {
        path.path.iter().any(|step| {
            matches!(step, AccessStep::Field(field) if self.is_protected_field(field))
        })
    }

    fn is_protected_field(&self, field: &FieldId) -> bool {
        // Check if field is protected by invariants or other safety mechanisms
        false // Simplified for now
    }

    fn get_leaked_field(&self, path: &AccessPath) -> FieldId {
        // Get the most specific field from the access path
        path.path.iter().find_map(|step| {
            if let AccessStep::Field(field) = step {
                Some(field.clone())
            } else {
                None
            }
        }).unwrap_or_else(|| FieldId {
            module_name: String::new(),
            struct_name: String::new(),
            field_name: String::new(),
        })
    }

    fn get_context_field(&self, context: &ReferenceContext) -> FieldId {
        // Get the most relevant field from context
        context.access_paths.last()
            .and_then(|path| self.get_leaked_field(path))
            .unwrap_or_else(|| FieldId {
                module_name: String::new(),
                struct_name: String::new(),
                field_name: String::new(),
            })
    }

    fn reference_escapes_scope(&self, var: &str, context: &ReferenceContext) -> bool {
        matches!(context.current_state,
            ReferenceState::Borrowed { .. } |
            ReferenceState::Moved { .. }
        )
    }

    fn get_alias_source(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Variable(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn get_expression_aliases(&self, expr: &Expression) -> HashSet<String> {
        let mut aliases = HashSet::new();
        if let Some(source) = self.get_alias_source(expr) {
            aliases.insert(source);
        }
        aliases
    }
}

impl ScopeContext {
    fn new(level: usize) -> Self {
        Self {
            level,
            references: HashSet::new(),
            borrows: HashMap::new(),
            conditions: Vec::new(),
        }
    }
} 