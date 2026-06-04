/// A ternary digit: Negative (-1), Zero (0), or Positive (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Trit {
    Negative = -1,
    Zero = 0,
    Positive = 1,
}

impl Trit {
    /// Create a Trit from an i8 value. Returns `None` if the value is not -1, 0, or 1.
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::Negative),
            0 => Some(Trit::Zero),
            1 => Some(Trit::Positive),
            _ => None,
        }
    }

    /// Convert to i8.
    pub fn as_i8(self) -> i8 {
        self as i8
    }

    /// Parse from a character: '-' → Negative, '0' → Zero, '+' → Positive.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '-' => Some(Trit::Negative),
            '0' => Some(Trit::Zero),
            '+' => Some(Trit::Positive),
            _ => None,
        }
    }

    /// Convert to display character.
    pub fn as_char(self) -> char {
        match self {
            Trit::Negative => '-',
            Trit::Zero => '0',
            Trit::Positive => '+',
        }
    }

    /// Negate the trit: Positive ↔ Negative, Zero stays Zero.
    pub fn negate(self) -> Self {
        match self {
            Trit::Negative => Trit::Positive,
            Trit::Zero => Trit::Zero,
            Trit::Positive => Trit::Negative,
        }
    }

    /// All three trit values.
    pub fn all() -> [Trit; 3] {
        [Trit::Negative, Trit::Zero, Trit::Positive]
    }
}

impl std::fmt::Display for Trit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_i8_valid() {
        assert_eq!(Trit::from_i8(-1), Some(Trit::Negative));
        assert_eq!(Trit::from_i8(0), Some(Trit::Zero));
        assert_eq!(Trit::from_i8(1), Some(Trit::Positive));
    }

    #[test]
    fn test_from_i8_invalid() {
        assert_eq!(Trit::from_i8(2), None);
        assert_eq!(Trit::from_i8(-2), None);
        assert_eq!(Trit::from_i8(127), None);
    }

    #[test]
    fn test_from_char_roundtrip() {
        for t in Trit::all() {
            assert_eq!(Trit::from_char(t.as_char()), Some(t));
        }
    }

    #[test]
    fn test_negate() {
        assert_eq!(Trit::Positive.negate(), Trit::Negative);
        assert_eq!(Trit::Negative.negate(), Trit::Positive);
        assert_eq!(Trit::Zero.negate(), Trit::Zero);
    }
}
