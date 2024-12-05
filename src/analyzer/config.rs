#[derive(Debug, Clone, Default)]
pub struct AnalyzerConfig {
    pub strict_mode: bool,
    pub check_transfer_safety: bool,
    pub check_capability_safety: bool,
    pub check_shared_objects: bool,
    pub check_arithmetic_safety: bool,
    pub check_timestamp_safety: bool,
    pub check_id_verification: bool,
    pub check_type_safety: bool,
    
    pub max_loop_depth: usize,
    pub max_vector_depth: usize,
    pub max_external_calls_per_loop: usize,
    pub max_call_stack_depth: usize,
    
    pub max_gas_per_function: u64,
    pub max_gas_per_loop: u64,
    pub warn_gas_threshold: u64,
    
    pub max_module_size: usize,
    pub ignore_tests: bool,
}

impl AnalyzerConfig {
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            check_transfer_safety: true,
            check_capability_safety: true,
            check_shared_objects: true,
            check_arithmetic_safety: true,
            check_timestamp_safety: true,
            check_id_verification: true,
            check_type_safety: true,
            
            max_loop_depth: 3,
            max_vector_depth: 2,
            max_external_calls_per_loop: 1,
            max_call_stack_depth: 8,
            
            max_gas_per_function: 1_000_000,
            max_gas_per_loop: 100_000,
            warn_gas_threshold: 500_000,
            
            max_module_size: 10000,
            ignore_tests: false,
        }
    }
} 