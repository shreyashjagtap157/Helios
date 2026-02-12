use crate::ir::{IrFunction, IrModule};
use crate::codegen::jit::{CompiledMethod, CompilationTier};
use crate::codegen::native_codegen::{NativeCodegen, TargetTriple};
use crate::codegen::opt::{OptimizationPass, dce::DeadCodeElimination, const_prop::ConstantPropagation};

/// Optimizing JIT Compiler
/// 
/// Uses profile data to perform advanced optimizations like:
/// - Inlining
/// - Loop unrolling
/// - Vectorization
/// - Constant propagation with assumptions
pub struct OptimizingJit {
    optimization_level: u8,
    passes: Vec<Box<dyn OptimizationPass + Send + Sync>>,
}

impl OptimizingJit {
    pub fn new() -> Self {
        let mut passes: Vec<Box<dyn OptimizationPass + Send + Sync>> = Vec::new();
        passes.push(Box::new(ConstantPropagation));
        passes.push(Box::new(DeadCodeElimination));
        passes.push(Box::new(crate::codegen::opt::licm::LoopInvariantCodeMotion));
        
        OptimizingJit {
            optimization_level: 2,
            passes,
        }
    }

    pub fn compile(&self, func: &IrFunction) -> Result<CompiledMethod, String> {
        // 1. Clone function for mutation
        let mut optimized_func = func.clone();
        
        // 2. Run optimization passes
        let mut changed = true;
        let max_iterations = 10;
        let mut iter = 0;
        
        while changed && iter < max_iterations {
            changed = false;
            for pass in &self.passes {
                if pass.run(&mut optimized_func) {
                    changed = true;
                }
            }
            iter += 1;
        }
        
        // 3. Code Generation
        // Wrap in a temporary module for NativeCodegen
        let wrapper_module = IrModule {
            name: format!("jit_opt_{}", func.name),
            functions: vec![optimized_func],
            globals: vec![],
            externs: vec![],
            vtables: vec![],
            string_pool: vec![],
            type_info: vec![],
        };
        
        let target = TargetTriple::host();
        let mut codegen = NativeCodegen::new(target);
        codegen.set_opt_level(self.optimization_level as u32);
        
        let output = codegen.compile_module(&wrapper_module)?;
        
        // 4. Extract compiled code
        let symbol = output.symbols.iter()
            .find(|s| s.name == func.name)
            .ok_or("Function symbol not found in generated code")?;
            
        Ok(CompiledMethod {
            name: func.name.clone(),
            tier: CompilationTier::Optimizing,
            machine_code: output.binary,
            code_offset: symbol.offset,
            code_size: symbol.size,
            deopt_info: vec![],
            stack_maps: vec![],
            ic_sites: vec![],
            compile_time_us: 0,
            assumptions: vec![],
        })
    }
}
