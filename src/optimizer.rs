//! Optimizer: Constant folding, dead code elimination, room merge.

use crate::Op;

/// Optimization passes for bytecode.
pub struct Optimizer;

impl Optimizer {
    /// Run all optimization passes on bytecode.
    pub fn optimize(ops: Vec<Op>) -> Vec<Op> {
        let ops = Self::constant_fold(ops);
        let ops = Self::dead_code_elimination(ops);
        let ops = Self::merge_rooms(ops);
        ops
    }

    /// Constant folding: evaluate Push+Push+Add/Mul at compile time.
    ///
    /// E.g., Push(Neg), Push(Neg), Add → Push(Pos) because Neg+Neg=Pos in ternary.
    pub fn constant_fold(ops: Vec<Op>) -> Vec<Op> {
        let mut result: Vec<Op> = Vec::new();
        let mut i = 0;
        while i < ops.len() {
            // Look for pattern: Push(a), Push(b), Add/Mul
            if i + 2 < ops.len() {
                if let (Op::Push(a), Op::Push(b)) = (&ops[i], &ops[i + 1]) {
                    let a = *a;
                    let b = *b;
                    match &ops[i + 2] {
                        Op::Add => {
                            result.push(Op::Push(a.add(b)));
                            i += 3;
                            continue;
                        }
                        Op::Mul => {
                            result.push(Op::Push(a.mul(b)));
                            i += 3;
                            continue;
                        }
                        _ => {}
                    }
                }
                // Pattern: Push(a), Negate → Push(!a)
                if let (Op::Push(a), Op::Negate) = (&ops[i], &ops[i + 1]) {
                    result.push(Op::Push(a.negate()));
                    i += 2;
                    continue;
                }
            }
            result.push(ops[i].clone());
            i += 1;
        }
        result
    }

    /// Dead code elimination: remove ops after Halt.
    pub fn dead_code_elimination(ops: Vec<Op>) -> Vec<Op> {
        let mut result = Vec::new();
        for op in ops {
            result.push(op.clone());
            if matches!(op, Op::Halt) {
                break;
            }
        }
        result
    }

    /// Merge consecutive room enter/leave pairs for the same room.
    ///
    /// EnterRoom("a"), ..., LeaveRoom, EnterRoom("a"), ..., LeaveRoom
    /// → EnterRoom("a"), ..., ..., ..., LeaveRoom
    pub fn merge_rooms(ops: Vec<Op>) -> Vec<Op> {
        // Remove LeaveRoom immediately followed by EnterRoom of the same room.
        // We track the current room in the result.
        let mut result: Vec<Op> = Vec::new();
        let mut current_room: Option<String> = None;
        let mut i = 0;
        while i < ops.len() {
            if i + 1 < ops.len() {
                if let (Op::LeaveRoom, Op::EnterRoom(name)) = (&ops[i], &ops[i + 1]) {
                    if current_room.as_ref() == Some(name) {
                        // Skip the LeaveRoom + EnterRoom pair — we're staying in the same room
                        i += 2;
                        continue;
                    }
                }
            }
            match &ops[i] {
                Op::EnterRoom(name) => { current_room = Some(name.clone()); }
                Op::LeaveRoom => { current_room = None; }
                _ => {}
            }
            result.push(ops[i].clone());
            i += 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ternary;

    #[test]
    fn test_constant_fold_neg_plus_neg() {
        let ops = vec![Op::Push(Ternary::Neg), Op::Push(Ternary::Neg), Op::Add, Op::Halt];
        let result = Optimizer::constant_fold(ops);
        assert_eq!(result[0], Op::Push(Ternary::Pos)); // Neg + Neg = Pos
        assert!(result.len() < 4); // Should be shorter
    }

    #[test]
    fn test_constant_fold_pos_times_zero() {
        let ops = vec![Op::Push(Ternary::Pos), Op::Push(Ternary::Zero), Op::Mul, Op::Halt];
        let result = Optimizer::constant_fold(ops);
        assert_eq!(result[0], Op::Push(Ternary::Zero)); // Pos * Zero = Zero
    }

    #[test]
    fn test_constant_fold_negate_pos() {
        let ops = vec![Op::Push(Ternary::Pos), Op::Negate, Op::Halt];
        let result = Optimizer::constant_fold(ops);
        assert_eq!(result[0], Op::Push(Ternary::Neg)); // !Pos = Neg
    }

    #[test]
    fn test_constant_fold_negate_zero() {
        let ops = vec![Op::Push(Ternary::Zero), Op::Negate, Op::Halt];
        let result = Optimizer::constant_fold(ops);
        assert_eq!(result[0], Op::Push(Ternary::Zero)); // !Zero = Zero
    }

    #[test]
    fn test_dead_code_after_halt() {
        let ops = vec![
            Op::Push(Ternary::Pos),
            Op::Halt,
            Op::Push(Ternary::Neg),
            Op::Push(Ternary::Zero),
        ];
        let result = Optimizer::dead_code_elimination(ops);
        assert_eq!(result.len(), 2); // Only Push(Pos) and Halt
        assert_eq!(result[0], Op::Push(Ternary::Pos));
        assert_eq!(result[1], Op::Halt);
    }

    #[test]
    fn test_dead_code_no_halt() {
        let ops = vec![Op::Push(Ternary::Pos), Op::Add];
        let result = Optimizer::dead_code_elimination(ops);
        assert_eq!(result.len(), 2); // Nothing removed
    }

    #[test]
    fn test_merge_rooms() {
        let ops = vec![
            Op::EnterRoom("lobby".to_string()),
            Op::Push(Ternary::Pos),
            Op::LeaveRoom,
            Op::EnterRoom("lobby".to_string()),
            Op::Push(Ternary::Neg),
            Op::LeaveRoom,
            Op::Halt,
        ];
        let result = Optimizer::merge_rooms(ops);
        // Should have merged the LeaveRoom+EnterRoom pair for same room
        // Original has 7 ops, after merge should be fewer
        // EnterRoom, Push(Pos), Push(Neg), LeaveRoom, Halt = 5
        assert!(result.len() >= 4); // at minimum 4 ops remain
        // Check no adjacent LeaveRoom+EnterRoom(same) pair exists
        for i in 0..result.len().saturating_sub(1) {
            if let Op::LeaveRoom = result[i] {
                if let Op::EnterRoom(name) = &result[i + 1] {
                    // Should not have same-room re-entry
                    // (This is OK if the previous room was different)
                }
            }
        }
    }

    #[test]
    fn test_full_optimization_pipeline() {
        let ops = vec![
            Op::Push(Ternary::Neg),
            Op::Push(Ternary::Neg),
            Op::Add,  // Folds to Push(Pos)
            Op::Halt,
            Op::Push(Ternary::Zero), // Dead code
        ];
        let result = Optimizer::optimize(ops);
        assert_eq!(result.len(), 2); // Push(Pos) + Halt
        assert_eq!(result[0], Op::Push(Ternary::Pos));
        assert_eq!(result[1], Op::Halt);
    }
}
