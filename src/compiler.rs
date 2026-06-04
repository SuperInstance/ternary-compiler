use crate::{StrategyIR, CompiledPolicy, Action, Optimizer};

/// The compiler converts a `StrategyIR` into a `CompiledPolicy`.
///
/// The compilation pipeline:
/// 1. Convert trits → actions
/// 2. Run optimization passes (dead-code elimination, constant folding)
/// 3. Build the lookup table
pub struct Compiler {
    optimizer: Optimizer,
}

/// Configuration for the compiler.
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Run dead-code elimination pass.
    pub dead_code_elimination: bool,
    /// Run constant folding pass.
    pub constant_folding: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            dead_code_elimination: true,
            constant_folding: true,
        }
    }
}

impl Compiler {
    /// Create a new compiler with default configuration.
    pub fn new() -> Self {
        Self {
            optimizer: Optimizer::new(),
        }
    }

    /// Create a compiler with custom configuration.
    pub fn with_config(config: CompilerConfig) -> Self {
        Self {
            optimizer: Optimizer::with_flags(config.dead_code_elimination, config.constant_folding),
        }
    }

    /// Compile a `StrategyIR` into a `CompiledPolicy`.
    ///
    /// This is the main entry point: strategy IR → optimized lookup table.
    pub fn compile(&self, ir: &StrategyIR) -> CompiledPolicy {
        let original_count = ir.len();
        let name = ir.metadata.name.clone();

        // Step 1: Convert trits to actions
        let mut actions: Vec<Action> = ir.trits().iter().map(|&t| Action::from_trit(t)).collect();

        // Step 2: Run optimization passes
        let stable_flags = ir.stable_flags();
        self.optimizer.optimize(&mut actions, stable_flags);

        // Step 3: Build and return the compiled policy
        CompiledPolicy::new(name, actions, original_count)
    }

    /// Compile without any optimization passes (raw translation).
    pub fn compile_raw(ir: &StrategyIR) -> CompiledPolicy {
        let original_count = ir.len();
        let actions: Vec<Action> = ir.trits().iter().map(|&t| Action::from_trit(t)).collect();
        CompiledPolicy::new(ir.metadata.name.clone(), actions, original_count)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Trit;

    #[test]
    fn test_compile_basic() {
        let ir = StrategyIR::from_trits("test", &[Trit::Positive, Trit::Negative, Trit::Zero]);
        let compiler = Compiler::new();
        let policy = compiler.compile(&ir);

        assert_eq!(policy.action(0), Some(Action::Commit));
        assert_eq!(policy.action(1), Some(Action::Oppose));
        assert_eq!(policy.action(2), Some(Action::Neutral));
    }

    #[test]
    fn test_compile_raw() {
        let ir = StrategyIR::from_trits("test", &[Trit::Positive]);
        let policy = Compiler::compile_raw(&ir);
        assert_eq!(policy.action(0), Some(Action::Commit));
        assert_eq!(policy.eliminated_count(), 0);
    }

    #[test]
    fn test_compile_with_stable_elimination() {
        let mut ir = StrategyIR::from_trits("test", &[Trit::Zero, Trit::Positive, Trit::Zero]);
        // Mark zero positions as stable → optimizer should eliminate them
        ir.set_stable(0, true);
        ir.set_stable(2, true);

        let compiler = Compiler::new();
        let policy = compiler.compile(&ir);

        assert_eq!(policy.action(0), Some(Action::Eliminated));
        assert_eq!(policy.action(1), Some(Action::Commit));
        assert_eq!(policy.action(2), Some(Action::Eliminated));
        assert_eq!(policy.eliminated_count(), 2);
    }

    #[test]
    fn test_compile_no_optimization() {
        let config = CompilerConfig {
            dead_code_elimination: false,
            constant_folding: false,
        };
        let compiler = Compiler::with_config(config);
        let mut ir = StrategyIR::from_trits("test", &[Trit::Zero, Trit::Positive]);
        ir.set_stable(0, true);

        let policy = compiler.compile(&ir);
        assert_eq!(policy.action(0), Some(Action::Neutral));
    }

    #[test]
    fn test_compile_empty() {
        let ir = StrategyIR::new("empty");
        let compiler = Compiler::new();
        let policy = compiler.compile(&ir);
        assert!(policy.is_empty());
    }
}
