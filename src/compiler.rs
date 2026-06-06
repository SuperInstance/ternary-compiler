//! Compiler: Compile AST to bytecode.

use crate::{Bytecode, Op, Ternary};
use crate::ast::{TernaryExpr, RoomDef, GateDef};

/// Compiles AST nodes into bytecode operations.
pub struct Compiler {
    bytecode: Bytecode,
}

/// Compile result.
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub bytecode: Bytecode,
    pub room_count: usize,
    pub op_count: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            bytecode: Bytecode::new(Vec::new()),
        }
    }

    /// Compile a full AST expression into bytecode.
    pub fn compile(&mut self, expr: &TernaryExpr) -> CompileResult {
        self.bytecode = Bytecode::new(Vec::new());
        self.compile_expr(expr);
        self.bytecode.ops.push(Op::Halt);

        // Collect rooms
        self.collect_rooms(expr);

        // Collect constants
        self.collect_constants();

        let op_count = self.bytecode.ops.len();
        let room_count = self.bytecode.rooms.len();
        CompileResult {
            bytecode: std::mem::replace(&mut self.bytecode, Bytecode::new(Vec::new())),
            room_count,
            op_count,
        }
    }

    fn compile_expr(&mut self, expr: &TernaryExpr) {
        match expr {
            TernaryExpr::Lit(t) => {
                self.bytecode.ops.push(Op::Push(*t));
            }
            TernaryExpr::Var(_name) => {
                // Variables are looked up at runtime — push a zero placeholder
                // In a full implementation, we'd have a Load op
                self.bytecode.ops.push(Op::Push(Ternary::Zero));
            }
            TernaryExpr::Add(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.bytecode.ops.push(Op::Add);
            }
            TernaryExpr::Mul(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.bytecode.ops.push(Op::Mul);
            }
            TernaryExpr::Negate(e) => {
                self.compile_expr(e);
                self.bytecode.ops.push(Op::Negate);
            }
            TernaryExpr::Branch(cond, if_pos, if_neg) => {
                // Branch: evaluate condition, then branch
                // For simplicity, compile as: cond; branch(pos_label, neg_label); pos_code; merge; neg_code; merge
                self.compile_expr(cond);
                let pos_label = format!("branch_pos_{}", self.bytecode.ops.len());
                let neg_label = format!("branch_neg_{}", self.bytecode.ops.len());
                self.bytecode.ops.push(Op::Branch(pos_label.clone(), neg_label.clone()));
                // Positive path
                self.compile_expr(if_pos);
                self.bytecode.ops.push(Op::Merge);
                // Negative path (we don't have labels, so the VM handles this by checking stack)
                self.compile_expr(if_neg);
                self.bytecode.ops.push(Op::Merge);
            }
            TernaryExpr::Sequence(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
            }
            TernaryExpr::Parallel(a, b) => {
                // Parallel: compile both, results stay on stack
                self.compile_expr(a);
                self.compile_expr(b);
                // Merge results
                self.bytecode.ops.push(Op::Merge);
            }
            TernaryExpr::Room(room_def) => {
                self.compile_room(room_def);
            }
            TernaryExpr::Passage(p) => {
                // Passage: enter from-room, traverse, leave
                self.bytecode.ops.push(Op::EnterRoom(p.from.clone()));
                self.bytecode.ops.push(Op::Push(Ternary::Zero)); // passage traversal value
                self.bytecode.ops.push(Op::LeaveRoom);
                self.bytecode.ops.push(Op::EnterRoom(p.to.clone()));
                self.bytecode.ops.push(Op::Push(Ternary::Zero));
                self.bytecode.ops.push(Op::LeaveRoom);
            }
            TernaryExpr::Gate(gate_def) => {
                self.compile_gate(gate_def);
            }
            TernaryExpr::Block(block) => {
                for e in &block.exprs {
                    self.compile_expr(e);
                }
            }
        }
    }

    fn compile_room(&mut self, room: &RoomDef) {
        self.bytecode.ops.push(Op::EnterRoom(room.name.clone()));
        self.compile_expr(&room.body);
        self.bytecode.ops.push(Op::LeaveRoom);
    }

    fn compile_gate(&mut self, gate: &GateDef) {
        self.compile_expr(&gate.condition);
        let pos_label = format!("gate_{}_pos", gate.name);
        let neg_label = format!("gate_{}_neg", gate.name);
        self.bytecode.ops.push(Op::Branch(pos_label.clone(), neg_label.clone()));
        // Pos path
        self.compile_expr(&gate.if_pos);
        self.bytecode.ops.push(Op::Merge);
        // Zero path (between pos and neg)
        self.compile_expr(&gate.if_zero);
        self.bytecode.ops.push(Op::Merge);
        // Neg path
        self.compile_expr(&gate.if_neg);
        self.bytecode.ops.push(Op::Merge);
    }

    fn collect_rooms(&mut self, expr: &TernaryExpr) {
        match expr {
            TernaryExpr::Room(r) => {
                if !self.bytecode.rooms.contains(&r.name) {
                    self.bytecode.rooms.push(r.name.clone());
                }
                self.collect_rooms(&r.body);
            }
            TernaryExpr::Passage(p) => {
                for r in [&p.from, &p.to] {
                    if !self.bytecode.rooms.contains(r) {
                        self.bytecode.rooms.push(r.clone());
                    }
                }
            }
            TernaryExpr::Add(a, b) | TernaryExpr::Mul(a, b) |
            TernaryExpr::Sequence(a, b) | TernaryExpr::Parallel(a, b) => {
                self.collect_rooms(a);
                self.collect_rooms(b);
            }
            TernaryExpr::Negate(e) | TernaryExpr::Branch(_, e, _) => {
                // Note: Branch has 3 children but we'll handle it above in the match
                self.collect_rooms(e);
            }
            TernaryExpr::Gate(g) => {
                self.collect_rooms(&g.condition);
                self.collect_rooms(&g.if_neg);
                self.collect_rooms(&g.if_zero);
                self.collect_rooms(&g.if_pos);
            }
            TernaryExpr::Block(b) => {
                for e in &b.exprs {
                    self.collect_rooms(e);
                }
            }
            TernaryExpr::Lit(_) | TernaryExpr::Var(_) => {}
        }
    }

    fn collect_constants(&mut self) {
        for op in &self.bytecode.ops {
            if let Op::Push(t) = op {
                if !self.bytecode.constants.contains(t) {
                    self.bytecode.constants.push(*t);
                }
            }
        }
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
    use crate::lexer::Lexer;

    fn compile_source(src: &str) -> CompileResult {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = crate::ast::Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(&ast)
    }

    #[test]
    fn test_compile_literal_pos() {
        let result = compile_source("pos");
        assert!(result.bytecode.ops.contains(&Op::Push(Ternary::Pos)));
        assert!(result.bytecode.ops.iter().any(|op| matches!(op, Op::Halt)));
    }

    #[test]
    fn test_compile_addition() {
        let result = compile_source("neg + neg");
        let ops = &result.bytecode.ops;
        assert_eq!(ops[0], Op::Push(Ternary::Neg));
        assert_eq!(ops[1], Op::Push(Ternary::Neg));
        assert_eq!(ops[2], Op::Add);
    }

    #[test]
    fn test_compile_multiplication() {
        let result = compile_source("pos * zero");
        let ops = &result.bytecode.ops;
        assert_eq!(ops[0], Op::Push(Ternary::Pos));
        assert_eq!(ops[1], Op::Push(Ternary::Zero));
        assert_eq!(ops[2], Op::Mul);
    }

    #[test]
    fn test_compile_negation() {
        let result = compile_source("!pos");
        let ops = &result.bytecode.ops;
        assert_eq!(ops[0], Op::Push(Ternary::Pos));
        assert_eq!(ops[1], Op::Negate);
    }

    #[test]
    fn test_compile_room() {
        let result = compile_source("room start { pos }");
        let ops = &result.bytecode.ops;
        assert_eq!(ops[0], Op::EnterRoom("start".to_string()));
        assert_eq!(ops[1], Op::Push(Ternary::Pos));
        assert_eq!(ops[2], Op::LeaveRoom);
        assert!(result.bytecode.rooms.contains(&"start".to_string()));
    }

    #[test]
    fn test_compile_ends_with_halt() {
        let result = compile_source("zero");
        let last = result.bytecode.ops.last().unwrap();
        assert_eq!(*last, Op::Halt);
    }
}
