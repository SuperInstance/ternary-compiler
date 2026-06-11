//! Ternary logic compiler backend

/// Ternary trit value: False(-1), Unknown(0), True(1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trit {
    False = -1,
    Unknown = 0,
    True = 1,
}

impl Trit {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::False),
            0 => Some(Trit::Unknown),
            1 => Some(Trit::True),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

/// A compiled ternary instruction
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: OpCode,
    pub operands: Vec<Trit>,
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    And,
    Or,
    Not,
    Consensus,
    Load,
    Store,
    Halt,
}

/// Compiler result
pub struct CompilerResult {
    pub instructions: Vec<Instruction>,
    pub symbol_table: std::collections::HashMap<String, usize>,
}

/// Compile source into ternary instructions
pub fn compile(_source: &str) -> Result<CompilerResult, String> {
    Ok(CompilerResult {
        instructions: vec![Instruction { opcode: OpCode::Halt, operands: vec![] }],
        symbol_table: std::collections::HashMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trit_roundtrip() {
        assert_eq!(Trit::from_i8(Trit::True.to_i8()), Some(Trit::True));
        assert_eq!(Trit::from_i8(Trit::Unknown.to_i8()), Some(Trit::Unknown));
        assert_eq!(Trit::from_i8(Trit::False.to_i8()), Some(Trit::False));
    }
}
