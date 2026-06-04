use crate::{CompiledPolicy, Action, Trit, StrategyIR};

/// The disassembler converts a `CompiledPolicy` back into human-readable text.
pub struct Disassembler;

impl Disassembler {
    /// Disassemble a compiled policy into a human-readable strategy string.
    ///
    /// Each position is represented as:
    /// - `+` for Commit
    /// - `-` for Oppose
    /// - `0` for Neutral
    /// - `x` for Eliminated
    pub fn to_text(policy: &CompiledPolicy) -> String {
        policy
            .actions()
            .iter()
            .map(|&action| match action {
                Action::Commit => '+',
                Action::Oppose => '-',
                Action::Neutral => '0',
                Action::Eliminated => 'x',
            })
            .collect()
    }

    /// Disassemble into a detailed multi-line report.
    pub fn to_detailed(policy: &CompiledPolicy) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Policy: {}", policy.name()));
        lines.push(format!(
            "Positions: {} active / {} total ({} eliminated)",
            policy.active_count(),
            policy.original_count(),
            policy.eliminated_count(),
        ));

        if policy.original_count() > 0 {
            lines.push(format!(
                "Compression: {:.1}%",
                policy.compression_ratio() * 100.0
            ));
        }

        lines.push(String::new());
        lines.push("idx  action     trit  code".to_string());
        lines.push("─".repeat(30).to_string());

        for (i, action) in policy.iter() {
            let trit_str = action
                .to_trit()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "—".to_string());
            lines.push(format!(
                "{:>3}  {:<10} {:>4}   {}",
                i,
                action,
                trit_str,
                action.short_code(),
            ));
        }

        lines.join("\n")
    }

    /// Convert a compiled policy back into a `StrategyIR` (lossy: eliminated
    /// positions become Zero trits, all positions are marked unstable).
    pub fn to_ir(policy: &CompiledPolicy) -> StrategyIR {
        let trits: Vec<Trit> = policy
            .actions()
            .iter()
            .map(|&action| action.to_trit().unwrap_or(Trit::Zero))
            .collect();

        let mut ir = StrategyIR::from_trits(policy.name(), &trits);
        // Mark eliminated positions as stable-zero
        for (i, &action) in policy.actions().iter().enumerate() {
            if action == Action::Eliminated {
                ir.set_stable(i, true);
            }
        }
        ir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_text() {
        let policy = CompiledPolicy::new(
            "test",
            vec![Action::Commit, Action::Oppose, Action::Neutral, Action::Eliminated],
            4,
        );
        assert_eq!(Disassembler::to_text(&policy), "+-0x");
    }

    #[test]
    fn test_to_detailed() {
        let policy = CompiledPolicy::new(
            "test",
            vec![Action::Commit, Action::Oppose],
            2,
        );
        let detailed = Disassembler::to_detailed(&policy);
        assert!(detailed.contains("Policy: test"));
        assert!(detailed.contains("commit"));
        assert!(detailed.contains("oppose"));
    }

    #[test]
    fn test_to_ir_roundtrip() {
        let original_trits = vec![Trit::Positive, Trit::Negative, Trit::Zero];
        let ir = StrategyIR::from_trits("test", &original_trits);
        let policy = crate::Compiler::compile_raw(&ir);
        let recovered = Disassembler::to_ir(&policy);

        assert_eq!(recovered.trits(), original_trits.as_slice());
    }

    #[test]
    fn test_to_ir_with_eliminated() {
        let policy = CompiledPolicy::new(
            "test",
            vec![Action::Commit, Action::Eliminated],
            2,
        );
        let ir = Disassembler::to_ir(&policy);
        assert_eq!(ir.trit(0), Some(Trit::Positive));
        assert_eq!(ir.trit(1), Some(Trit::Zero)); // eliminated → zero
        assert!(ir.is_stable(1)); // eliminated positions are stable
    }
}
