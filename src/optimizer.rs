use crate::Action;

/// The optimizer runs passes on the action table before final compilation.
///
/// Supported passes:
/// - **Dead-code elimination**: removes stable neutral (zero) positions
/// - **Constant folding**: collapses consecutive identical actions into a single representative
pub struct Optimizer {
    enable_dead_code_elimination: bool,
    enable_constant_folding: bool,
}

impl Optimizer {
    /// Create an optimizer with all passes enabled.
    pub fn new() -> Self {
        Self {
            enable_dead_code_elimination: true,
            enable_constant_folding: true,
        }
    }

    /// Create an optimizer with specific pass flags.
    pub fn with_flags(dead_code_elimination: bool, constant_folding: bool) -> Self {
        Self {
            enable_dead_code_elimination: dead_code_elimination,
            enable_constant_folding: constant_folding,
        }
    }

    /// Run all enabled optimization passes on the action table.
    ///
    /// `stable_flags` indicates which positions are known-stable in the original strategy.
    pub fn optimize(&self, actions: &mut [Action], stable_flags: &[bool]) {
        if self.enable_dead_code_elimination {
            self.dead_code_elimination(actions, stable_flags);
        }
        if self.enable_constant_folding {
            self.constant_folding(actions);
        }
    }

    /// **Dead-code elimination**: positions that are both neutral and stable
    /// carry no useful information and can be eliminated.
    ///
    /// A position is "dead" if:
    /// - Its action is `Neutral`, AND
    /// - It is marked as stable (won't change)
    fn dead_code_elimination(&self, actions: &mut [Action], stable_flags: &[bool]) {
        for (i, action) in actions.iter_mut().enumerate() {
            let is_stable = stable_flags.get(i).copied().unwrap_or(false);
            if *action == Action::Neutral && is_stable {
                *action = Action::Eliminated;
            }
        }
    }

    /// **Constant folding**: when a sequence of positions all map to the same action,
    /// we don't fold them (lookup tables need per-position entries) — but we do
    /// fold "Eliminated" chains by marking trailing Eliminated positions.
    ///
    /// In practice, constant folding here means: if a Neutral action is surrounded
    /// by two identical non-neutral actions, fold it to match its neighbors.
    /// This reduces table indirection for "obviously determined" positions.
    fn constant_folding(&self, actions: &mut [Action]) {
        if actions.len() < 3 {
            return;
        }

        // Forward pass: if a Neutral has the same non-Neutral neighbor on both sides, fold it.
        let len = actions.len();
        for i in 1..len.saturating_sub(1) {
            if actions[i] == Action::Neutral {
                let prev = actions[i - 1];
                let next = actions[i + 1];
                if prev == next && prev != Action::Neutral && prev != Action::Eliminated {
                    actions[i] = prev;
                }
            }
        }
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dead_code_elimination() {
        let mut actions = vec![Action::Neutral, Action::Commit, Action::Neutral];
        let stable = vec![true, false, true];
        let opt = Optimizer::with_flags(true, false);
        opt.optimize(&mut actions, &stable);
        assert_eq!(actions[0], Action::Eliminated);
        assert_eq!(actions[1], Action::Commit);
        assert_eq!(actions[2], Action::Eliminated);
    }

    #[test]
    fn test_dead_code_only_neutral_stable() {
        let mut actions = vec![Action::Neutral, Action::Commit];
        let stable = vec![false, true]; // Neutral is NOT stable
        let opt = Optimizer::with_flags(true, false);
        opt.optimize(&mut actions, &stable);
        assert_eq!(actions[0], Action::Neutral); // not eliminated
        assert_eq!(actions[1], Action::Commit);
    }

    #[test]
    fn test_constant_folding() {
        let mut actions = vec![Action::Commit, Action::Neutral, Action::Commit];
        let stable = vec![false, false, false];
        let opt = Optimizer::with_flags(false, true);
        opt.optimize(&mut actions, &stable);
        assert_eq!(actions[1], Action::Commit); // folded to match neighbors
    }

    #[test]
    fn test_constant_folding_no_match() {
        let mut actions = vec![Action::Commit, Action::Neutral, Action::Oppose];
        let stable = vec![false, false, false];
        let opt = Optimizer::with_flags(false, true);
        opt.optimize(&mut actions, &stable);
        assert_eq!(actions[1], Action::Neutral); // neighbors differ, no fold
    }

    #[test]
    fn test_constant_folding_too_short() {
        let mut actions = vec![Action::Commit, Action::Neutral];
        let stable = vec![];
        let opt = Optimizer::with_flags(false, true);
        opt.optimize(&mut actions, &stable);
        assert_eq!(actions[0], Action::Commit);
        assert_eq!(actions[1], Action::Neutral);
    }

    #[test]
    fn test_both_passes() {
        let mut actions = vec![Action::Neutral, Action::Commit, Action::Neutral, Action::Commit, Action::Neutral];
        let stable = vec![true, false, false, false, false];
        let opt = Optimizer::new();
        opt.optimize(&mut actions, &stable);
        // Position 0: Neutral + stable → Eliminated (dead code)
        assert_eq!(actions[0], Action::Eliminated);
        // Position 2: Neutral between Commit and Commit → folded to Commit
        assert_eq!(actions[2], Action::Commit);
        // Position 4: Neutral with no right neighbor, stays Neutral
        assert_eq!(actions[4], Action::Neutral);
    }
}
