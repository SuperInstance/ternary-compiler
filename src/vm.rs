//! VM: Simple stack-based VM that executes ternary bytecode.

use crate::{Op, Ternary};

/// Stack-based virtual machine for executing ternary bytecode.
#[derive(Debug, Clone)]
pub struct VM {
    /// Value stack.
    pub stack: Vec<Ternary>,
    /// Current room context.
    pub room: Option<String>,
    /// Program counter.
    pub pc: usize,
    /// Room history for tracking enter/leave.
    pub room_history: Vec<String>,
}

/// VM execution result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VMResult {
    /// Completed successfully with final value.
    Ok(Ternary),
    /// Hit a branch point.
    Branched { if_pos: usize, if_neg: usize },
    /// Hit halt.
    Halted(Ternary),
    /// Error during execution.
    Error(String),
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            room: None,
            pc: 0,
            room_history: Vec::new(),
        }
    }

    /// Execute a full bytecode program, returning the top-of-stack result.
    pub fn run(&mut self, ops: &[Op]) -> VMResult {
        self.stack.clear();
        self.pc = 0;
        self.room = None;
        self.room_history.clear();

        while self.pc < ops.len() {
            match &ops[self.pc] {
                Op::Push(t) => {
                    self.stack.push(*t);
                    self.pc += 1;
                }
                Op::Add => {
                    let b = self.stack.pop().unwrap_or(Ternary::Zero);
                    let a = self.stack.pop().unwrap_or(Ternary::Zero);
                    self.stack.push(a.add(b));
                    self.pc += 1;
                }
                Op::Mul => {
                    let b = self.stack.pop().unwrap_or(Ternary::Zero);
                    let a = self.stack.pop().unwrap_or(Ternary::Zero);
                    self.stack.push(a.mul(b));
                    self.pc += 1;
                }
                Op::Negate => {
                    let a = self.stack.pop().unwrap_or(Ternary::Zero);
                    self.stack.push(a.negate());
                    self.pc += 1;
                }
                Op::EnterRoom(name) => {
                    self.room = Some(name.clone());
                    self.room_history.push(name.clone());
                    self.pc += 1;
                }
                Op::LeaveRoom => {
                    self.room = None;
                    self.pc += 1;
                }
                Op::Branch(_pos_label, _neg_label) => {
                    let cond = self.stack.pop().unwrap_or(Ternary::Zero);
                    match cond {
                        Ternary::Pos => {
                            // Skip to after the first merge (positive path was compiled first)
                            self.pc += 1;
                            // Execute until merge
                            let _start_pc = self.pc;
                            let mut merge_count = 0;
                            while self.pc < ops.len() {
                                match &ops[self.pc] {
                                    Op::Merge => {
                                        merge_count += 1;
                                        if merge_count == 1 {
                                            self.pc += 1;
                                            break;
                                        }
                                    }
                                    Op::Halt => break,
                                    _ => {}
                                }
                                self.execute_single(ops);
                            }
                        }
                        Ternary::Neg => {
                            // Skip past positive path and its merge, then execute negative path
                            self.pc += 1;
                            let mut merge_count = 0;
                            while self.pc < ops.len() {
                                match &ops[self.pc] {
                                    Op::Merge => {
                                        merge_count += 1;
                                        if merge_count == 2 {
                                            self.pc += 1;
                                            break;
                                        }
                                    }
                                    Op::Halt => break,
                                    _ => {}
                                }
                                self.pc += 1;
                            }
                        }
                        Ternary::Zero => {
                            // Zero: skip positive path, execute zero path (between first and second merge)
                            self.pc += 1;
                            let mut merge_count = 0;
                            while self.pc < ops.len() {
                                match &ops[self.pc] {
                                    Op::Merge => {
                                        merge_count += 1;
                                        if merge_count == 1 {
                                            self.pc += 1;
                                            break;
                                        }
                                    }
                                    Op::Halt => break,
                                    _ => {}
                                }
                                self.pc += 1;
                            }
                        }
                    }
                }
                Op::Merge => {
                    // No-op at runtime — just skip
                    self.pc += 1;
                }
                Op::Halt => {
                    let val = self.stack.last().copied().unwrap_or(Ternary::Zero);
                    return VMResult::Halted(val);
                }
            }
        }

        let val = self.stack.last().copied().unwrap_or(Ternary::Zero);
        VMResult::Ok(val)
    }

    /// Execute a single operation at the current PC (for Branch paths).
    fn execute_single(&mut self, ops: &[Op]) {
        match &ops[self.pc] {
            Op::Push(t) => {
                self.stack.push(*t);
                self.pc += 1;
            }
            Op::Add => {
                let b = self.stack.pop().unwrap_or(Ternary::Zero);
                let a = self.stack.pop().unwrap_or(Ternary::Zero);
                self.stack.push(a.add(b));
                self.pc += 1;
            }
            Op::Mul => {
                let b = self.stack.pop().unwrap_or(Ternary::Zero);
                let a = self.stack.pop().unwrap_or(Ternary::Zero);
                self.stack.push(a.mul(b));
                self.pc += 1;
            }
            Op::Negate => {
                let a = self.stack.pop().unwrap_or(Ternary::Zero);
                self.stack.push(a.negate());
                self.pc += 1;
            }
            Op::EnterRoom(name) => {
                self.room = Some(name.clone());
                self.room_history.push(name.clone());
                self.pc += 1;
            }
            Op::LeaveRoom => {
                self.room = None;
                self.pc += 1;
            }
            _ => {
                self.pc += 1;
            }
        }
    }

    /// Get the current top-of-stack value.
    pub fn top(&self) -> Option<Ternary> {
        self.stack.last().copied()
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_ops(ops: &[Op]) -> Ternary {
        let mut vm = VM::new();
        match vm.run(ops) {
            VMResult::Ok(t) | VMResult::Halted(t) => t,
            VMResult::Error(e) => panic!("VM error: {}", e),
            _ => panic!("Unexpected branch result"),
        }
    }

    #[test]
    fn test_vm_push() {
        let ops = vec![Op::Push(Ternary::Pos), Op::Halt];
        assert_eq!(run_ops(&ops), Ternary::Pos);
    }

    #[test]
    fn test_vm_add_neg_neg() {
        // Neg + Neg = Pos (wrapping)
        let ops = vec![Op::Push(Ternary::Neg), Op::Push(Ternary::Neg), Op::Add, Op::Halt];
        assert_eq!(run_ops(&ops), Ternary::Pos);
    }

    #[test]
    fn test_vm_add_pos_pos() {
        // Pos + Pos = Neg (wrapping)
        let ops = vec![Op::Push(Ternary::Pos), Op::Push(Ternary::Pos), Op::Add, Op::Halt];
        assert_eq!(run_ops(&ops), Ternary::Neg);
    }

    #[test]
    fn test_vm_add_neg_pos() {
        let ops = vec![Op::Push(Ternary::Neg), Op::Push(Ternary::Pos), Op::Add, Op::Halt];
        assert_eq!(run_ops(&ops), Ternary::Zero);
    }

    #[test]
    fn test_vm_mul_pos_zero() {
        let ops = vec![Op::Push(Ternary::Pos), Op::Push(Ternary::Zero), Op::Mul, Op::Halt];
        assert_eq!(run_ops(&ops), Ternary::Zero);
    }

    #[test]
    fn test_vm_mul_neg_pos() {
        let ops = vec![Op::Push(Ternary::Neg), Op::Push(Ternary::Pos), Op::Mul, Op::Halt];
        assert_eq!(run_ops(&ops), Ternary::Neg);
    }

    #[test]
    fn test_vm_negate_pos() {
        let ops = vec![Op::Push(Ternary::Pos), Op::Negate, Op::Halt];
        assert_eq!(run_ops(&ops), Ternary::Neg);
    }

    #[test]
    fn test_vm_negate_zero() {
        let ops = vec![Op::Push(Ternary::Zero), Op::Negate, Op::Halt];
        assert_eq!(run_ops(&ops), Ternary::Zero);
    }

    #[test]
    fn test_vm_room_enter_leave() {
        let ops = vec![
            Op::EnterRoom("lobby".to_string()),
            Op::Push(Ternary::Pos),
            Op::LeaveRoom,
            Op::Halt,
        ];
        let mut vm = VM::new();
        vm.run(&ops);
        assert_eq!(vm.room, None); // left room
        assert_eq!(vm.room_history, vec!["lobby".to_string()]);
    }

    #[test]
    fn test_vm_room_context() {
        let ops = vec![
            Op::EnterRoom("hall".to_string()),
            Op::Push(Ternary::Neg),
            Op::LeaveRoom,
            Op::Halt,
        ];
        let mut vm = VM::new();
        // Check room context during execution isn't possible directly,
        // but we can check history after
        let _result = vm.run(&ops);
        assert_eq!(vm.room_history.len(), 1);
        assert_eq!(vm.room_history[0], "hall");
    }

    #[test]
    fn test_vm_complex_expression() {
        // (Neg + Neg) * Pos = Pos * Pos = Pos
        // But: Neg + Neg = Pos (wrapping), then Pos * Pos = Pos
        let ops = vec![
            Op::Push(Ternary::Neg),
            Op::Push(Ternary::Neg),
            Op::Add,
            Op::Push(Ternary::Pos),
            Op::Mul,
            Op::Halt,
        ];
        assert_eq!(run_ops(&ops), Ternary::Pos);
    }

    #[test]
    fn test_vm_empty_program() {
        let ops = vec![Op::Halt];
        assert_eq!(run_ops(&ops), Ternary::Zero);
    }
}
