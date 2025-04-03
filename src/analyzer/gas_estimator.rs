use super::parser::{Statement, Expression, Function};

#[derive(Debug)]
pub struct GasEstimator {
    pub loop_cost: u64,
    pub external_call_cost: u64,
    pub vector_op_cost: u64,
    pub storage_op_cost: u64,
    total_gas: u64,
}

#[derive(Debug, Clone)]
pub struct GasEstimate {
    pub base_cost: u64,
    pub max_cost: u64,
    pub operations: Vec<GasOperation>,
}

#[derive(Debug, Clone)]
pub struct GasOperation {
    pub operation_type: OperationType,
    pub cost: u64,
    pub location: String,
}

#[derive(Debug, Clone)]
pub enum OperationType {
    ExternalCall,
    LoopIteration,
    VectorOperation,
    StorageAccess,
    Computation,
}

#[derive(Debug, Clone)]
pub enum GasCost {
    Computation,
    Storage,
    Borrow,
    Call,
}

impl GasEstimator {
    pub fn new() -> Self {
        Self {
            loop_cost: 50,
            external_call_cost: 500,
            vector_op_cost: 100,
            storage_op_cost: 200,
            total_gas: 0,
        }
    }

    pub fn estimate_function_cost(&mut self, function: &Function) -> GasEstimate {
        let mut operations = Vec::new();
        let mut base_cost = 0u64;
        let mut max_cost = 0u64;
        let mut in_loop = false;

        for statement in &function.body {
            match statement {
                Statement::Loop(_) => {
                    in_loop = true;
                    base_cost += self.loop_cost;
                    max_cost += self.loop_cost * 10;
                    
                    operations.push(GasOperation {
                        operation_type: OperationType::LoopIteration,
                        cost: self.loop_cost,
                        location: format!("Function: {}, Loop", function.name),
                    });
                }
                Statement::ExternalCall(name) | Statement::Call(name, _) => {
                    let (cost, op_type) = if name.contains("vector::") {
                        (self.vector_op_cost, OperationType::VectorOperation)
                    } else {
                        (self.external_call_cost, OperationType::ExternalCall)
                    };
                    
                    let multiplier = if in_loop { 10 } else { 1 };
                    base_cost += cost;
                    max_cost += cost * multiplier;
                    
                    operations.push(GasOperation {
                        operation_type: op_type,
                        cost: cost * multiplier,
                        location: format!("Function: {}, Call: {}", function.name, name),
                    });
                }
                Statement::Assert(_) => {
                    let assertion_cost = 50;
                    base_cost += assertion_cost;
                    max_cost += assertion_cost;
                    
                    operations.push(GasOperation {
                        operation_type: OperationType::Computation,
                        cost: assertion_cost,
                        location: format!("Function: {}, Assertion", function.name),
                    });
                }
                Statement::Assignment(_, expr) => {
                    let (cost, op_type) = self.estimate_expression_cost(expr);
                    let multiplier = if in_loop { 5 } else { 1 };
                    base_cost += cost;
                    max_cost += cost * multiplier;
                    
                    operations.push(GasOperation {
                        operation_type: op_type,
                        cost: cost * multiplier,
                        location: format!("Function: {}, Assignment", function.name),
                    });
                }
                Statement::Return(expr) => {
                    let (cost, op_type) = self.estimate_expression_cost(expr);
                    base_cost += cost;
                    max_cost += cost;
                    
                    operations.push(GasOperation {
                        operation_type: op_type,
                        cost,
                        location: format!("Function: {}, Return", function.name),
                    });
                }
                Statement::InternalCall(_) => {
                    // Internal calls are handled separately
                }
                Statement::BorrowField(_) | Statement::BorrowGlobal(_) | Statement::BorrowLocal(_) => {
                    self.add_gas_cost(GasCost::Borrow);
                }
            }
        }

        GasEstimate {
            base_cost,
            max_cost,
            operations,
        }
    }

    fn estimate_expression_cost(&self, expr: &Expression) -> (u64, OperationType) {
        match expr {
            Expression::Variable(_) => (10, OperationType::Computation),
            Expression::FieldAccess(_, _) => (self.storage_op_cost, OperationType::StorageAccess),
            Expression::Call(name, args) => {
                let base_cost = if name.starts_with("Self::") {
                    100
                } else if name.contains("vector::") {
                    self.vector_op_cost
                } else {
                    self.external_call_cost
                };
                (base_cost + (args.len() as u64 * 10), OperationType::ExternalCall)
            }
            Expression::Value(_) => (5, OperationType::Computation),
        }
    }

    pub fn add_gas_cost(&mut self, cost: GasCost) {
        let amount = match cost {
            GasCost::Computation => 10,
            GasCost::Storage => 100,
            GasCost::Borrow => 20,
            GasCost::Call => 50,
        };
        self.total_gas += amount;
    }
} 