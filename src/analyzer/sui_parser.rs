use super::parser::{Parser, Module, Function, Statement, Expression, Type};
use super::types::*;
use std::collections::HashSet;

pub struct SuiMoveParser {
    base_parser: Parser,
    object_types: HashSet<String>,
    capability_types: HashSet<String>,
    shared_objects: HashSet<String>,
    transfer_functions: HashSet<String>,
}

impl SuiMoveParser {
    pub fn new(source: &str) -> Self {
        Self {
            base_parser: Parser::new(source),
            object_types: HashSet::new(),
            capability_types: HashSet::new(),
            shared_objects: HashSet::new(),
            transfer_functions: HashSet::new(),
        }
    }

    pub fn parse_module(&mut self) -> Result<Module, String> {
        let mut module = self.base_parser.parse()?;
        
        // Analyze and track Sui-specific types and patterns
        self.analyze_module_types(&mut module)?;
        
        // Enhance module with Sui-specific information
        self.enhance_module_info(&mut module)?;
        
        Ok(module)
    }

    fn analyze_module_types(&mut self, module: &Module) -> Result<(), String> {
        // Analyze struct definitions
        for struct_def in &module.structs {
            // Check for object types
            if self.is_object_type(&struct_def.name) {
                self.object_types.insert(struct_def.name.clone());
                
                // Check if it's a shared object
                if self.is_shared_object_type(&struct_def.name) {
                    self.shared_objects.insert(struct_def.name.clone());
                }
            }
            
            // Check for capability types
            if self.is_capability_type(&struct_def.name) {
                self.capability_types.insert(struct_def.name.clone());
            }
        }

        // Analyze functions
        for function in &module.functions {
            if self.is_transfer_function(function) {
                self.transfer_functions.insert(function.name.clone());
            }
        }

        Ok(())
    }

    fn enhance_module_info(&mut self, module: &mut Module) -> Result<(), String> {
        // Add Sui-specific attributes to types
        for struct_def in &mut module.structs {
            if self.object_types.contains(&struct_def.name) {
                // Add object-specific fields if missing
                self.ensure_object_fields(struct_def)?;
            }
            
            if self.capability_types.contains(&struct_def.name) {
                // Add capability-specific fields if missing
                self.ensure_capability_fields(struct_def)?;
            }
        }

        // Enhance functions with Sui-specific information
        for function in &mut module.functions {
            if self.transfer_functions.contains(&function.name) {
                // Add transfer-specific checks and guards
                self.enhance_transfer_function(function)?;
            }
        }

        Ok(())
    }

    fn ensure_object_fields(&self, struct_def: &mut Struct) -> Result<(), String> {
        let required_fields = ["id", "owner"];
        let mut missing_fields = Vec::new();

        for field in required_fields.iter() {
            if !struct_def.fields.iter().any(|f| f.name == *field) {
                missing_fields.push(*field);
            }
        }

        if !missing_fields.is_empty() {
            return Err(format!(
                "Object type {} missing required fields: {:?}",
                struct_def.name, missing_fields
            ));
        }

        Ok(())
    }

    fn ensure_capability_fields(&self, struct_def: &mut Struct) -> Result<(), String> {
        // Verify capability has proper structure
        if !struct_def.fields.is_empty() {
            return Err(format!(
                "Capability type {} should not have fields",
                struct_def.name
            ));
        }

        Ok(())
    }

    fn enhance_transfer_function(&self, function: &mut Function) -> Result<(), String> {
        // Check for required transfer parameters
        let has_recipient = function.parameters.iter().any(|p| {
            matches!(p.param_type, Type::Base(ref t) if t == "address")
        });

        if !has_recipient {
            return Err(format!(
                "Transfer function {} missing recipient parameter",
                function.name
            ));
        }

        Ok(())
    }

    fn is_object_type(&self, type_name: &str) -> bool {
        type_name.ends_with("Object") || 
        type_name.contains("::object::") ||
        self.has_key_ability(type_name)
    }

    fn is_capability_type(&self, type_name: &str) -> bool {
        type_name.ends_with("Cap") || 
        type_name.contains("::capability::") ||
        self.has_store_ability(type_name)
    }

    fn is_shared_object_type(&self, type_name: &str) -> bool {
        type_name.starts_with("shared::") || 
        type_name.contains("::shared::") ||
        self.has_shared_ability(type_name)
    }

    fn is_transfer_function(&self, function: &Function) -> bool {
        function.name.contains("transfer") ||
        self.has_transfer_pattern(function)
    }

    fn has_key_ability(&self, type_name: &str) -> bool {
        // Check if type has 'key' ability in source
        false // Simplified for now
    }

    fn has_store_ability(&self, type_name: &str) -> bool {
        // Check if type has 'store' ability in source
        false // Simplified for now
    }

    fn has_shared_ability(&self, type_name: &str) -> bool {
        // Check if type has shared object pattern
        false // Simplified for now
    }

    fn has_transfer_pattern(&self, function: &Function) -> bool {
        // Check if function contains transfer pattern
        function.body.iter().any(|stmt| {
            matches!(stmt, Statement::Assignment(_, Expression::FieldAccess(_, field)) 
                if field == "transfer" || field == "public_transfer")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_object_type() {
        let source = r#"
            module 0x1::test {
                struct TestObject has key {
                    id: ID,
                    owner: address,
                    value: u64,
                }
            }
        "#;

        let mut parser = SuiMoveParser::new(source);
        let module = parser.parse_module().unwrap();
        assert!(parser.object_types.contains("TestObject"));
    }

    #[test]
    fn test_parse_transfer_function() {
        let source = r#"
            module 0x1::test {
                public fun transfer(obj: TestObject, recipient: address) {
                    transfer::transfer(obj, recipient);
                }
            }
        "#;

        let mut parser = SuiMoveParser::new(source);
        let module = parser.parse_module().unwrap();
        assert!(parser.transfer_functions.contains("transfer"));
    }
} 