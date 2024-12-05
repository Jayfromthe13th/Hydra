use super::types::*;
use std::collections::HashSet;

pub struct Parser;

impl Parser {
    pub fn parse_module(source: &str) -> Result<Module, String> {
        Module::parse(source)
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub imports: HashSet<Import>,
    pub functions: Vec<Function>,
    pub structs: Vec<Struct>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Import {
    pub full_path: String,    // e.g., "sui::object"
    pub module_name: String,  // e.g., "object"
    pub members: Vec<String>, // e.g., ["Self", "UID"]
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assert(Expression),
    Loop(Expression),
    Assignment(String, Expression),
    Return(Expression),
    Call(String, Vec<Expression>),
    ExternalCall(String),
    InternalCall(String),
    BorrowField(String),
    BorrowGlobal(String),
    BorrowLocal(String),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Variable(String),
    FieldAccess(Box<Expression>, String),
    Call(String, Vec<Expression>),
    Value(String),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub is_public: bool,
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
    pub return_type: Option<Type>,
    pub has_loops: bool,
    pub has_assertions: bool,
    pub external_calls: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
    pub abilities: Vec<String>,
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
}

impl Module {
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut module = Module {
            name: String::new(),
            imports: HashSet::new(),
            functions: Vec::new(),
            structs: Vec::new(),
        };

        let mut current_function: Option<Function> = None;
        let mut in_function = false;
        let mut brace_count = 0;

        for line in source.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Parse module name
            if line.starts_with("module") {
                module.name = line.split("::").last()
                    .unwrap_or("")
                    .trim_end_matches("{")
                    .trim()
                    .to_string();
                continue;
            }

            // Parse imports
            if line.starts_with("use") {
                if let Some(import) = parse_import(line) {
                    module.imports.insert(import);
                }
                continue;
            }

            // Track braces
            brace_count += line.matches('{').count();
            brace_count -= line.matches('}').count();

            // Parse function start
            if line.contains("fun ") {
                let is_public = line.starts_with("public");
                let name = line.split("fun ")
                    .nth(1)
                    .and_then(|s| s.split('(').next())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                current_function = Some(Function {
                    name,
                    is_public,
                    parameters: Vec::new(),
                    body: Vec::new(),
                    return_type: None,
                    has_loops: false,
                    has_assertions: false,
                    external_calls: HashSet::new(),
                });
                in_function = true;
                continue;
            }

            // Parse function body
            if in_function {
                if let Some(ref mut func) = current_function {
                    // Check for assertions
                    if line.contains("assert!") {
                        func.body.push(Statement::Assert(Expression::Value("".to_string())));
                        func.has_assertions = true;
                    }

                    // Check for loops
                    if line.contains("while ") || line.contains("for ") {
                        func.body.push(Statement::Loop(Expression::Value("".to_string())));
                        func.has_loops = true;
                    }

                    // Check for external calls
                    if line.contains("::") {
                        let call = line.split("::")
                            .take(2)
                            .collect::<Vec<_>>()
                            .join("::");
                        
                        if let Some(call_name) = call.split(['(', ' ', ';']).next() {
                            if call_name.starts_with("Self::") {
                                func.body.push(Statement::InternalCall(call_name.to_string()));
                            } else {
                                func.body.push(Statement::ExternalCall(call_name.to_string()));
                                func.external_calls.insert(call_name.to_string());
                            }
                        }
                    }
                }

                // Function end
                if brace_count == 0 {
                    if let Some(func) = current_function.take() {
                        module.functions.push(func);
                    }
                    in_function = false;
                }
            }
        }

        Ok(module)
    }

    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: Vec::new(),
            imports: HashSet::new(),
            structs: Vec::new(),
        }
    }

    pub fn get_fields(&self) -> Vec<&Field> {
        // Return all fields from all structs
        self.structs.iter()
            .flat_map(|s| &s.fields)
            .collect()
    }

    pub fn get_structs(&self) -> &[Struct] {
        &self.structs
    }

    pub fn get_globals(&self) -> Vec<&Field> {
        self.structs.iter()
            .filter(|s| s.abilities.contains(&"global".to_string()))
            .flat_map(|s| &s.fields)
            .collect()
    }
}

fn parse_import(line: &str) -> Option<Import> {
    let line = line.trim_start_matches("use ").trim_end_matches(';');
    let parts: Vec<&str> = line.split("::").collect();
    
    if parts.is_empty() {
        return None;
    }

    let mut full_path = String::new();
    let mut module_name = String::new();
    let mut members = Vec::new();

    if parts[0].contains("0x1") {
        if parts.len() > 1 {
            full_path = format!("{}::{}", parts[0], parts[1]);
            module_name = parts[1].to_string();
        }
    } else {
        full_path = parts[0].to_string();
        module_name = parts[0].to_string();
    }

    if let Some(last) = parts.last() {
        if last.contains('{') {
            let member_list = last.trim_matches(|c| c == '{' || c == '}');
            members = member_list.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
    }

    Some(Import {
        full_path,
        module_name,
        members,
    })
}

#[allow(dead_code)]
fn parse_struct(line: &str, brace_count: &mut i32) -> Result<Option<Struct>, String> {
    let line = line.trim_start_matches("struct ").trim();
    let parts: Vec<&str> = line.split("has").collect();
    
    let name = parts[0].trim().to_string();
    let abilities = if parts.len() > 1 {
        parts[1].split_whitespace()
            .take_while(|&s| s != "{")
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        Vec::new()
    };

    *brace_count += line.matches('{').count() as i32;
    
    Ok(Some(Struct {
        name,
        fields: Vec::new(),
        abilities,
        attributes: Vec::new(),
    }))
}

impl Struct {
    pub fn has_key_ability(&self) -> bool {
        self.abilities.contains(&"key".to_string())
    }

    pub fn is_public(&self) -> bool {
        self.abilities.contains(&"public".to_string())
    }

    pub fn has_invariant(&self) -> bool {
        // Check for invariant annotations
        self.attributes.iter().any(|attr| attr.contains("invariant"))
    }

    pub fn get_invariant(&self) -> Option<String> {
        self.attributes.iter()
            .find(|attr| attr.contains("invariant"))
            .map(|attr| attr.to_string())
    }
}

impl Parameter {
    pub fn is_mutable_reference(&self) -> bool {
        matches!(self.param_type, Type::MutableReference(_))
    }
}

impl Field {
    pub fn is_public(&self) -> bool {
        // Add implementation
        true
    }

    pub fn has_invariant(&self) -> bool {
        // Add implementation
        false
    }

    pub fn get_invariant(&self) -> Option<String> {
        // Add implementation
        None
    }
}