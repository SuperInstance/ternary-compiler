use crate::{Trit, MAX_POSITIONS};

/// Metadata about a strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyMetadata {
    /// Human-readable name for this strategy.
    pub name: String,
    /// Version of the strategy format.
    pub version: u32,
    /// Number of positions in the strategy.
    pub position_count: usize,
}

impl Default for StrategyMetadata {
    fn default() -> Self {
        Self {
            name: String::from("unnamed"),
            version: 1,
            position_count: 0,
        }
    }
}

/// Information about a specific position in the strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionInfo {
    /// Index of this position.
    pub index: usize,
    /// The trit value at this position.
    pub trit: Trit,
    /// Whether this position is "stable" (won't change during optimization).
    pub stable: bool,
    /// Optional label for the position.
    pub label: Option<String>,
}

/// Intermediate representation of a ternary strategy.
///
/// A strategy is a sequence of trits (ternary digits) with metadata describing
/// how each position should behave. This IR is the input to the compiler.
#[derive(Debug, Clone)]
pub struct StrategyIR {
    /// The trit values for each position.
    trits: Vec<Trit>,
    /// Stability flags: `true` means the position is known-stable.
    stable: Vec<bool>,
    /// Labels for named positions.
    labels: Vec<Option<String>>,
    /// Strategy metadata.
    pub metadata: StrategyMetadata,
}

impl StrategyIR {
    /// Create a new empty strategy IR.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            trits: Vec::new(),
            stable: Vec::new(),
            labels: Vec::new(),
            metadata: StrategyMetadata {
                name: name.into(),
                version: 1,
                position_count: 0,
            },
        }
    }

    /// Create a strategy IR from a slice of trits. All positions start unstable.
    pub fn from_trits(name: impl Into<String>, trits: &[Trit]) -> Self {
        let len = trits.len().min(MAX_POSITIONS);
        Self {
            trits: trits[..len].to_vec(),
            stable: vec![false; len],
            labels: vec![None; len],
            metadata: StrategyMetadata {
                name: name.into(),
                version: 1,
                position_count: len,
            },
        }
    }

    /// Parse a strategy from text like "-0+0-+". Unrecognized chars are skipped.
    pub fn parse(name: impl Into<String>, text: &str) -> Self {
        let trits: Vec<Trit> = text
            .chars()
            .filter_map(Trit::from_char)
            .take(MAX_POSITIONS)
            .collect();
        let len = trits.len();
        Self {
            trits,
            stable: vec![false; len],
            labels: vec![None; len],
            metadata: StrategyMetadata {
                name: name.into(),
                version: 1,
                position_count: len,
            },
        }
    }

    /// Push a trit to the strategy. Returns `false` if at max capacity.
    pub fn push(&mut self, trit: Trit, stable: bool) -> bool {
        if self.trits.len() >= MAX_POSITIONS {
            return false;
        }
        self.trits.push(trit);
        self.stable.push(stable);
        self.labels.push(None);
        self.metadata.position_count = self.trits.len();
        true
    }

    /// Set the label for a position.
    pub fn set_label(&mut self, index: usize, label: impl Into<String>) {
        if index < self.labels.len() {
            self.labels[index] = Some(label.into());
        }
    }

    /// Mark a position as stable.
    pub fn set_stable(&mut self, index: usize, stable: bool) {
        if index < self.stable.len() {
            self.stable[index] = stable;
        }
    }

    /// Get the trit at a position.
    pub fn trit(&self, index: usize) -> Option<Trit> {
        self.trits.get(index).copied()
    }

    /// Check if a position is stable.
    pub fn is_stable(&self, index: usize) -> bool {
        self.stable.get(index).copied().unwrap_or(false)
    }

    /// Get the label for a position.
    pub fn label(&self, index: usize) -> Option<&str> {
        self.labels.get(index).and_then(|l| l.as_deref())
    }

    /// Number of positions.
    pub fn len(&self) -> usize {
        self.trits.len()
    }

    /// Whether the strategy is empty.
    pub fn is_empty(&self) -> bool {
        self.trits.is_empty()
    }

    /// Get all position info.
    pub fn positions(&self) -> Vec<PositionInfo> {
        self.trits
            .iter()
            .enumerate()
            .map(|(i, &trit)| PositionInfo {
                index: i,
                trit,
                stable: self.stable[i],
                label: self.labels[i].clone(),
            })
            .collect()
    }

    /// Get a reference to the trits slice.
    pub fn trits(&self) -> &[Trit] {
        &self.trits
    }

    /// Get a reference to the stable flags.
    pub fn stable_flags(&self) -> &[bool] {
        &self.stable
    }

    /// Serialize to text representation (e.g., "-0+0-+").
    pub fn to_text(&self) -> String {
        self.trits.iter().map(|t| t.as_char()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_strategy() {
        let s = StrategyIR::new("test");
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
        assert_eq!(s.metadata.name, "test");
    }

    #[test]
    fn test_from_trits() {
        let s = StrategyIR::from_trits("test", &[Trit::Negative, Trit::Zero, Trit::Positive]);
        assert_eq!(s.len(), 3);
        assert_eq!(s.trit(0), Some(Trit::Negative));
        assert_eq!(s.trit(1), Some(Trit::Zero));
        assert_eq!(s.trit(2), Some(Trit::Positive));
        assert_eq!(s.trit(3), None);
    }

    #[test]
    fn test_parse_text() {
        let s = StrategyIR::parse("test", "-0+0-+");
        assert_eq!(s.len(), 6);
        assert_eq!(s.to_text(), "-0+0-+");
    }

    #[test]
    fn test_parse_ignores_bad_chars() {
        let s = StrategyIR::parse("test", "- x 0 +");
        assert_eq!(s.len(), 3);
        assert_eq!(s.to_text(), "-0+");
    }

    #[test]
    fn test_push_and_labels() {
        let mut s = StrategyIR::new("test");
        assert!(s.push(Trit::Positive, true));
        assert!(s.push(Trit::Negative, false));
        s.set_label(0, "start");
        assert_eq!(s.label(0), Some("start"));
        assert!(s.is_stable(0));
        assert!(!s.is_stable(1));
    }

    #[test]
    fn test_positions() {
        let s = StrategyIR::from_trits("test", &[Trit::Zero, Trit::Positive]);
        let positions = s.positions();
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].trit, Trit::Zero);
        assert_eq!(positions[1].trit, Trit::Positive);
    }
}
