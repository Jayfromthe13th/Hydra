use std::collections::{HashMap, HashSet};
use crate::analyzer::parser::{Module, Statement};

#[derive(Debug, Clone)]
pub enum InvariantKind {
    Field(String),
    Global(String),
    Struct(String),
}

#[derive(Debug)]
pub struct InvariantInfo {
    pub kind: InvariantKind,
    pub condition: String,
    pub fields: HashSet<String>,
    pub globals: HashSet<String>,
}

#[derive(Debug)]
pub struct InvariantTracker {
    field_invariants: HashMap<String, InvariantInfo>,
    global_invariants: HashMap<String, InvariantInfo>,
    struct_invariants: HashMap<String, InvariantInfo>,
    current_module: Option<String>,
    global_state: HashMap<String, Vec<String>>,
}

impl InvariantTracker {
    pub fn new() -> Self {
        Self {
            field_invariants: HashMap::new(),
            global_invariants: HashMap::new(),
            struct_invariants: HashMap::new(),
            current_module: None,
            global_state: HashMap::new(),
        }
    }

    pub fn parse_invariants(&mut self, module: &Module) -> Result<(), String> {
        self.current_module = Some(module.name.clone());

        // Parse field invariants
        for field in module.get_fields() {
            if let Some(invariant) = field.get_invariant() {
                self.field_invariants.insert(
                    field.name.clone(),
                    InvariantInfo {
                        kind: InvariantKind::Field(field.name.clone()),
                        condition: invariant.clone(),
                        fields: HashSet::new(),
                        globals: HashSet::new(),
                    }
                );
            }
        }

        // Parse global invariants
        for global in module.get_globals() {
            if let Some(invariant) = global.get_invariant() {
                self.global_invariants.insert(
                    global.name.clone(),
                    InvariantInfo {
                        kind: InvariantKind::Global(global.name.clone()),
                        condition: invariant.clone(),
                        fields: HashSet::new(),
                        globals: HashSet::new(),
                    }
                );
            }
        }

        // Parse struct invariants
        for struct_def in module.get_structs() {
            if let Some(invariant) = struct_def.get_invariant() {
                self.struct_invariants.insert(
                    struct_def.name.clone(),
                    InvariantInfo {
                        kind: InvariantKind::Struct(struct_def.name.clone()),
                        condition: invariant.clone(),
                        fields: struct_def.fields.iter().map(|f| f.name.clone()).collect(),
                        globals: HashSet::new(),
                    }
                );
            }
        }

        Ok(())
    }

    pub fn track_global_state(&mut self, statement: &Statement) {
        match statement {
            Statement::BorrowGlobal(type_name) => {
                if let Some(module) = &self.current_module {
                    self.global_state
                        .entry(module.clone())
                        .or_default()
                        .push(type_name.clone());
                }
            }
            _ => {}
        }
    }

    pub fn get_field_invariants(&self, field_name: &str) -> Option<&InvariantInfo> {
        self.field_invariants.get(field_name)
    }

    pub fn get_global_invariants(&self, global_name: &str) -> Option<&InvariantInfo> {
        self.global_invariants.get(global_name)
    }

    pub fn get_struct_invariants(&self, struct_name: &str) -> Option<&InvariantInfo> {
        self.struct_invariants.get(struct_name)
    }

    pub fn get_all_invariants(&self) -> Vec<&InvariantInfo> {
        let mut all = Vec::new();
        all.extend(self.field_invariants.values());
        all.extend(self.global_invariants.values());
        all.extend(self.struct_invariants.values());
        all
    }

    pub fn has_invariant(&self, name: &str) -> bool {
        self.field_invariants.contains_key(name) ||
        self.global_invariants.contains_key(name) ||
        self.struct_invariants.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_invariant() {
        let mut tracker = InvariantTracker::new();
        let mut module = Module::new("test");
        
        // Add field with invariant
        module.add_field_with_invariant(
            "balance",
            "u64",
            "balance >= 0"
        );

        assert!(tracker.parse_invariants(&module).is_ok());
        assert!(tracker.has_invariant("balance"));
    }

    #[test]
    fn test_parse_struct_invariant() {
        let mut tracker = InvariantTracker::new();
        let mut module = Module::new("test");
        
        // Add struct with invariant
        module.add_struct_with_invariant(
            "Coin",
            vec![("value", "u64")],
            "value > 0"
        );

        assert!(tracker.parse_invariants(&module).is_ok());
        assert!(tracker.has_invariant("Coin"));
    }
} 