use crate::{CompiledPolicy, Action, Trit, MAX_ENVIRONMENTS};

/// Statistics for a single action path.
#[derive(Debug, Clone, PartialEq)]
pub struct PathStats {
    /// How many times this position was accessed.
    pub hits: usize,
    /// The action at this position.
    pub action: Action,
    /// The index of this position.
    pub index: usize,
}

impl Default for PathStats {
    fn default() -> Self {
        Self {
            hits: 0,
            action: Action::Neutral,
            index: 0,
        }
    }
}

/// A profiling report for a compiled policy.
#[derive(Debug, Clone)]
pub struct ProfileReport {
    /// Per-position statistics.
    pub paths: Vec<PathStats>,
    /// Total number of environment evaluations.
    pub total_evaluations: usize,
    /// Name of the profiled policy.
    pub policy_name: String,
}

impl ProfileReport {
    /// Number of hot paths (positions with hits above the given threshold).
    pub fn hot_paths(&self, threshold: usize) -> Vec<&PathStats> {
        self.paths
            .iter()
            .filter(|p| p.hits >= threshold)
            .collect()
    }

    /// Number of cold paths (positions with zero hits).
    pub fn cold_paths(&self) -> Vec<&PathStats> {
        self.paths.iter().filter(|p| p.hits == 0).collect()
    }

    /// Most-accessed position (if any).
    pub fn hottest_path(&self) -> Option<&PathStats> {
        self.paths.iter().max_by_key(|p| p.hits)
    }

    /// Least-accessed position that still had some access (if any).
    pub fn coldest_active_path(&self) -> Option<&PathStats> {
        self.paths
            .iter()
            .filter(|p| p.hits > 0)
            .min_by_key(|p| p.hits)
    }
}

/// An environment is a sequence of trits that the policy is evaluated against.
/// Each position in the environment "queries" the corresponding position in the policy.
pub type Environment = Vec<Trit>;

/// The profiler evaluates a compiled policy against environments and reports
/// which positions are hot (frequently accessed) or cold (rarely accessed).
pub struct Profiler {
    /// Access counts per position.
    counts: Vec<usize>,
}

impl Profiler {
    /// Create a new profiler for a given policy.
    pub fn new(policy: &CompiledPolicy) -> Self {
        Self {
            counts: vec![0; policy.len()],
        }
    }

    /// Profile the policy against a single environment.
    ///
    /// For each position, if the environment's trit aligns with the policy's action
    /// (e.g., Positive trit + Commit action), it counts as a "hit".
    pub fn evaluate(&mut self, policy: &CompiledPolicy, env: &Environment) {
        for i in 0..policy.len().min(env.len()) {
            let action = policy.action(i).unwrap_or(Action::Eliminated);
            if action == Action::Eliminated {
                continue;
            }
            // A "hit" occurs when the environment's trit direction matches the action
            let trit = env[i];
            let is_hit = match action {
                Action::Commit => trit == Trit::Positive,
                Action::Oppose => trit == Trit::Negative,
                Action::Neutral => trit == Trit::Zero,
                Action::Eliminated => false,
            };
            if is_hit {
                self.counts[i] += 1;
            }
        }
    }

    /// Profile the policy against multiple environments.
    pub fn evaluate_many(&mut self, policy: &CompiledPolicy, envs: &[Environment]) {
        for env in envs.iter().take(MAX_ENVIRONMENTS) {
            self.evaluate(policy, env);
        }
    }

    /// Generate the profiling report.
    pub fn report(self, policy: &CompiledPolicy) -> ProfileReport {
        let paths: Vec<PathStats> = policy
            .iter()
            .map(|(i, action)| PathStats {
                index: i,
                action,
                hits: self.counts.get(i).copied().unwrap_or(0),
            })
            .collect();

        ProfileReport {
            paths,
            total_evaluations: self.counts.iter().sum(),
            policy_name: policy.name().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_policy() -> CompiledPolicy {
        CompiledPolicy::new(
            "test",
            vec![Action::Commit, Action::Oppose, Action::Neutral],
            3,
        )
    }

    #[test]
    fn test_profiler_basic() {
        let policy = make_policy();
        let mut profiler = Profiler::new(&policy);

        // Environment matches all positions
        profiler.evaluate(&policy, &vec![Trit::Positive, Trit::Negative, Trit::Zero]);

        let report = profiler.report(&policy);
        assert_eq!(report.paths[0].hits, 1);
        assert_eq!(report.paths[1].hits, 1);
        assert_eq!(report.paths[2].hits, 1);
        assert_eq!(report.total_evaluations, 3);
    }

    #[test]
    fn test_profiler_mismatches() {
        let policy = make_policy();
        let mut profiler = Profiler::new(&policy);

        // None match
        profiler.evaluate(&policy, &vec![Trit::Negative, Trit::Positive, Trit::Positive]);

        let report = profiler.report(&policy);
        assert_eq!(report.total_evaluations, 0);
    }

    #[test]
    fn test_hot_and_cold() {
        let policy = make_policy();
        let mut profiler = Profiler::new(&policy);

        // Hit position 0 many times
        for _ in 0..10 {
            profiler.evaluate(&policy, &vec![Trit::Positive, Trit::Positive, Trit::Positive]);
        }
        let report = profiler.report(&policy);
        assert_eq!(report.hot_paths(5).len(), 1); // only position 0
        assert_eq!(report.cold_paths().len(), 2); // positions 1 and 2
        assert_eq!(report.hottest_path().unwrap().index, 0);
    }

    #[test]
    fn test_evaluate_many() {
        let policy = make_policy();
        let mut profiler = Profiler::new(&policy);

        let envs: Vec<Environment> = vec![
            vec![Trit::Positive, Trit::Negative, Trit::Zero],
            vec![Trit::Positive, Trit::Negative, Trit::Zero],
        ];
        profiler.evaluate_many(&policy, &envs);
        let report = profiler.report(&policy);
        assert_eq!(report.paths[0].hits, 2);
    }

    #[test]
    fn test_profiler_with_eliminated() {
        let policy = CompiledPolicy::new(
            "test",
            vec![Action::Commit, Action::Eliminated, Action::Neutral],
            3,
        );
        let mut profiler = Profiler::new(&policy);
        profiler.evaluate(&policy, &vec![Trit::Positive, Trit::Zero, Trit::Zero]);
        let report = profiler.report(&policy);
        assert_eq!(report.paths[0].hits, 1);
        assert_eq!(report.paths[1].hits, 0); // eliminated, never counted
        assert_eq!(report.paths[2].hits, 1);
    }
}
