//! # ternary-compiler
//!
//! Compiles ternary strategy descriptions into optimized lookup tables — the
//! "compiler" for the ternary runtime.
//!
//! ## Pipeline
//!
//! ```text
//! Strategy text → StrategyIR → Optimizer → Compiler → CompiledPolicy
//!                                                         ↕
//!                                               Profiler / Disassembler
//! ```

mod trit;
mod strategy_ir;
mod compiled_policy;
mod compiler;
mod optimizer;
mod profiler;
mod disassembler;

pub use trit::Trit;
pub use strategy_ir::{StrategyIR, StrategyMetadata, PositionInfo};
pub use compiled_policy::{CompiledPolicy, Action};
pub use compiler::Compiler;
pub use optimizer::Optimizer;
pub use profiler::{Profiler, ProfileReport, PathStats};
pub use disassembler::Disassembler;

/// Maximum number of positions in a strategy.
pub const MAX_POSITIONS: usize = 64;

/// Maximum number of environments for profiling.
pub const MAX_ENVIRONMENTS: usize = 256;
