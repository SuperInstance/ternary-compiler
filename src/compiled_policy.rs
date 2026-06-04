use crate::Trit;

/// Action to take at a given position in the compiled policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Actively commit to this position (positive stance).
    Commit,
    /// Actively oppose this position (negative stance).
    Oppose,
    /// Remain neutral / skip this position.
    Neutral,
    /// This position was eliminated by dead-code optimization.
    Eliminated,
}

impl Action {
    /// Convert a trit to the corresponding action (non-eliminated).
    pub fn from_trit(trit: Trit) -> Self {
        match trit {
            Trit::Positive => Action::Commit,
            Trit::Negative => Action::Oppose,
            Trit::Zero => Action::Neutral,
        }
    }

    /// Convert back to a trit, if possible. `Eliminated` has no trit.
    pub fn to_trit(self) -> Option<Trit> {
        match self {
            Action::Commit => Some(Trit::Positive),
            Action::Oppose => Some(Trit::Negative),
            Action::Neutral => Some(Trit::Zero),
            Action::Eliminated => None,
        }
    }

    /// Human-readable short code.
    pub fn short_code(self) -> &'static str {
        match self {
            Action::Commit => "C",
            Action::Oppose => "O",
            Action::Neutral => "N",
            Action::Eliminated => "X",
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Commit => write!(f, "commit"),
            Action::Oppose => write!(f, "oppose"),
            Action::Neutral => write!(f, "neutral"),
            Action::Eliminated => write!(f, "eliminated"),
        }
    }
}

/// An optimized lookup table compiled from a strategy IR.
///
/// Provides O(1) lookup from position index to action.
#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    /// The action table: index → action.
    table: Vec<Action>,
    /// Original position count before optimization.
    original_count: usize,
    /// Number of eliminated positions.
    eliminated_count: usize,
    /// Optional name carried from the strategy.
    name: String,
}

impl CompiledPolicy {
    /// Create a new compiled policy from actions.
    pub fn new(name: impl Into<String>, actions: Vec<Action>, original_count: usize) -> Self {
        let eliminated_count = actions.iter().filter(|a| **a == Action::Eliminated).count();
        Self {
            table: actions,
            original_count,
            eliminated_count,
            name: name.into(),
        }
    }

    /// Look up the action at a given index. O(1).
    pub fn action(&self, index: usize) -> Option<Action> {
        self.table.get(index).copied()
    }

    /// Total number of entries in the table.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Whether the policy is empty.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Number of positions that were eliminated by optimization.
    pub fn eliminated_count(&self) -> usize {
        self.eliminated_count
    }

    /// Original position count before optimization.
    pub fn original_count(&self) -> usize {
        self.original_count
    }

    /// Active (non-eliminated) position count.
    pub fn active_count(&self) -> usize {
        self.table.len() - self.eliminated_count
    }

    /// Name of the policy.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Iterate over all (index, action) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (usize, Action)> + '_ {
        self.table.iter().enumerate().map(|(i, &a)| (i, a))
    }

    /// Get a reference to the actions slice.
    pub fn actions(&self) -> &[Action] {
        &self.table
    }

    /// Count of each action type.
    pub fn action_counts(&self) -> ActionCounts {
        let mut counts = ActionCounts::default();
        for &action in &self.table {
            match action {
                Action::Commit => counts.commit += 1,
                Action::Oppose => counts.oppose += 1,
                Action::Neutral => counts.neutral += 1,
                Action::Eliminated => counts.eliminated += 1,
            }
        }
        counts
    }

    /// Compression ratio: active / original positions.
    pub fn compression_ratio(&self) -> f64 {
        if self.original_count == 0 {
            return 1.0;
        }
        self.active_count() as f64 / self.original_count as f64
    }
}

/// Counts of each action type in a compiled policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActionCounts {
    pub commit: usize,
    pub oppose: usize,
    pub neutral: usize,
    pub eliminated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_from_trit() {
        assert_eq!(Action::from_trit(Trit::Positive), Action::Commit);
        assert_eq!(Action::from_trit(Trit::Negative), Action::Oppose);
        assert_eq!(Action::from_trit(Trit::Zero), Action::Neutral);
    }

    #[test]
    fn test_action_to_trit_roundtrip() {
        assert_eq!(Action::Commit.to_trit(), Some(Trit::Positive));
        assert_eq!(Action::Eliminated.to_trit(), None);
    }

    #[test]
    fn test_compiled_policy_lookup() {
        let policy = CompiledPolicy::new(
            "test",
            vec![Action::Commit, Action::Oppose, Action::Neutral],
            3,
        );
        assert_eq!(policy.action(0), Some(Action::Commit));
        assert_eq!(policy.action(1), Some(Action::Oppose));
        assert_eq!(policy.action(2), Some(Action::Neutral));
        assert_eq!(policy.action(3), None);
    }

    #[test]
    fn test_action_counts() {
        let policy = CompiledPolicy::new(
            "test",
            vec![Action::Commit, Action::Oppose, Action::Neutral, Action::Eliminated],
            4,
        );
        let counts = policy.action_counts();
        assert_eq!(counts.commit, 1);
        assert_eq!(counts.oppose, 1);
        assert_eq!(counts.neutral, 1);
        assert_eq!(counts.eliminated, 1);
    }

    #[test]
    fn test_compression_ratio() {
        let policy = CompiledPolicy::new(
            "test",
            vec![Action::Commit, Action::Eliminated, Action::Neutral],
            3,
        );
        assert!((policy.compression_ratio() - (2.0 / 3.0)).abs() < f64::EPSILON);
    }
}
