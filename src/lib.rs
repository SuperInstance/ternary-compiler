//! # Ternary Compiler
//!
//! A compiler that converts ternary decisions `{-1, 0, +1}` into simple bytecode
//! for coordination. Takes ternary-mud room algebra and emits executable coordination instructions.
//!
//! ## Modules
//!
//! - **lexer** — Tokenize ternary expressions
//! - **ast** — Abstract syntax tree for ternary programs
//! - **compiler** — Compile AST to bytecode
//! - **vm** — Stack-based VM that executes bytecode
//! - **optimizer** — Constant folding, dead code elimination, room merge
//! - **ir** — Intermediate representation: BasicBlock, CFG, dominator tree

pub mod lexer;
pub mod ast;
pub mod compiler;
pub mod vm;
pub mod optimizer;
pub mod ir;

use serde::{Deserialize, Serialize};

/// Core ternary value: Negative (-1), Zero (0), or Positive (+1).
///
/// Ternary arithmetic rules:
/// - Addition wraps: (-1)+(-1)=+1, (+1)+(+1)=-1, (-1)+(+1)=0
/// - Multiplication: standard sign rules
/// - Negation: flips sign, zero stays zero
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ternary {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Ternary {
    /// Create from i8. Clamps to valid range.
    pub fn from_i8(v: i8) -> Self {
        match v {
            -1 => Ternary::Neg,
            0 => Ternary::Zero,
            1 => Ternary::Pos,
            _ => Ternary::Zero,
        }
    }

    /// Convert to i8.
    pub fn to_i8(self) -> i8 {
        self as i8
    }

    /// Ternary addition with wrapping: (-1)+(-1)=+1, (+1)+(+1)=-1.
    pub fn add(self, other: Ternary) -> Ternary {
        let raw = self.to_i8() + other.to_i8();
        // Wrap into {-1, 0, 1}: -2→+1, -1→-1, 0→0, 1→+1, 2→-1
        let wrapped = ((raw + 1).rem_euclid(3)) - 1;
        Ternary::from_i8(wrapped)
    }

    /// Ternary multiplication: standard sign rules.
    pub fn mul(self, other: Ternary) -> Ternary {
        Ternary::from_i8(self.to_i8() * other.to_i8())
    }

    /// Ternary negation: !Pos = Neg, !Neg = Pos, !Zero = Zero.
    pub fn negate(self) -> Ternary {
        match self {
            Ternary::Pos => Ternary::Neg,
            Ternary::Neg => Ternary::Pos,
            Ternary::Zero => Ternary::Zero,
        }
    }

    /// Parse from character: '-' => Neg, '0' => Zero, '+' => Pos.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '-' => Some(Ternary::Neg),
            '0' => Some(Ternary::Zero),
            '+' => Some(Ternary::Pos),
            _ => None,
        }
    }
}

/// Bytecode operations for the ternary VM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
    Push(Ternary),
    Add,
    Mul,
    Negate,
    EnterRoom(String),
    LeaveRoom,
    Branch(String, String),
    Merge,
    Halt,
}

/// Compiled bytecode program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bytecode {
    pub ops: Vec<Op>,
    pub constants: Vec<Ternary>,
    pub rooms: Vec<String>,
}

impl Bytecode {
    pub fn new(ops: Vec<Op>) -> Self {
        Bytecode {
            ops,
            constants: Vec::new(),
            rooms: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_add_neg_neg() {
        assert_eq!(Ternary::Neg.add(Ternary::Neg), Ternary::Pos);
    }

    #[test]
    fn test_ternary_add_pos_pos() {
        assert_eq!(Ternary::Pos.add(Ternary::Pos), Ternary::Neg);
    }

    #[test]
    fn test_ternary_add_neg_pos() {
        assert_eq!(Ternary::Neg.add(Ternary::Pos), Ternary::Zero);
    }

    #[test]
    fn test_ternary_mul_neg_pos() {
        assert_eq!(Ternary::Neg.mul(Ternary::Pos), Ternary::Neg);
    }

    #[test]
    fn test_ternary_mul_zero_anything() {
        assert_eq!(Ternary::Zero.mul(Ternary::Pos), Ternary::Zero);
        assert_eq!(Ternary::Zero.mul(Ternary::Neg), Ternary::Zero);
    }

    #[test]
    fn test_ternary_negate() {
        assert_eq!(Ternary::Pos.negate(), Ternary::Neg);
        assert_eq!(Ternary::Neg.negate(), Ternary::Pos);
        assert_eq!(Ternary::Zero.negate(), Ternary::Zero);
    }
}
