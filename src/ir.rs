//! IR: Intermediate representation — BasicBlock, ControlFlowGraph, dominator tree.

use crate::Op;
use std::collections::{HashMap, HashSet};

/// A basic block in the IR.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: String,
    pub ops: Vec<Op>,
    pub successors: Vec<String>,
}

/// Control flow graph built from bytecode.
#[derive(Debug, Clone)]
pub struct CFG {
    pub blocks: Vec<BasicBlock>,
    pub entry: String,
}

impl CFG {
    /// Build a CFG from a flat list of ops.
    ///
    /// Splits at branch points and halt instructions.
    pub fn from_ops(ops: &[Op]) -> Self {
        let mut blocks: Vec<BasicBlock> = Vec::new();
        let mut current_ops: Vec<Op> = Vec::new();
        let mut block_idx = 0;

        let entry = format!("bb{}", block_idx);

        for op in ops {
            match op {
                Op::Branch(pos_label, neg_label) => {
                    // End current block with the branch
                    let succs = vec![pos_label.clone(), neg_label.clone()];
                    current_ops.push(op.clone());

                    let label = format!("bb{}", block_idx);
                    blocks.push(BasicBlock {
                        label,
                        ops: std::mem::take(&mut current_ops),
                        successors: succs,
                    });
                    block_idx += 1;
                }
                Op::Halt => {
                    current_ops.push(op.clone());
                    let label = format!("bb{}", block_idx);
                    blocks.push(BasicBlock {
                        label,
                        ops: std::mem::take(&mut current_ops),
                        successors: vec![],
                    });
                    block_idx += 1;
                }
                Op::Merge => {
                    current_ops.push(op.clone());
                    let label = format!("bb{}", block_idx);
                    blocks.push(BasicBlock {
                        label,
                        ops: std::mem::take(&mut current_ops),
                        successors: vec![format!("bb{}", block_idx + 1)],
                    });
                    block_idx += 1;
                }
                _ => {
                    current_ops.push(op.clone());
                }
            }
        }

        // Flush remaining ops as a final block
        if !current_ops.is_empty() {
            let label = format!("bb{}", block_idx);
            blocks.push(BasicBlock {
                label,
                ops: current_ops,
                successors: vec![],
            });
        }

        // If no blocks were created (empty program), create a single empty entry block
        if blocks.is_empty() && !ops.is_empty() {
            blocks.push(BasicBlock {
                label: entry.clone(),
                ops: ops.to_vec(),
                successors: vec![],
            });
        }

        let entry_label = blocks.first().map(|b| b.label.clone()).unwrap_or_else(|| "bb0".to_string());

        CFG { blocks, entry: entry_label }
    }

    /// Get a block by label.
    pub fn get_block(&self, label: &str) -> Option<&BasicBlock> {
        self.blocks.iter().find(|b| b.label == label)
    }

    /// Get block count.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

/// Build a dominator tree from a CFG.
///
/// Returns a map: block label → set of dominators (including itself).
pub fn dominator_tree(cfg: &CFG) -> HashMap<String, HashSet<String>> {
    let block_labels: Vec<String> = cfg.blocks.iter().map(|b| b.label.clone()).collect();
    let all_labels: HashSet<String> = block_labels.iter().cloned().collect();

    // Initialize: entry dominated only by itself, all others by everything
    let mut dom: HashMap<String, HashSet<String>> = HashMap::new();
    for label in &block_labels {
        if label == &cfg.entry {
            dom.insert(label.clone(), std::iter::once(label.clone()).collect());
        } else {
            dom.insert(label.clone(), all_labels.clone());
        }
    }

    // Build predecessor map
    let mut preds: HashMap<String, HashSet<String>> = HashMap::new();
    for block in &cfg.blocks {
        for succ in &block.successors {
            preds.entry(succ.clone()).or_default().insert(block.label.clone());
        }
    }

    // Iterative dataflow
    let mut changed = true;
    while changed {
        changed = false;
        for label in &block_labels {
            if label == &cfg.entry {
                continue;
            }
            let pred_doms: Vec<&HashSet<String>> = preds
                .get(label)
                .map(|ps| ps.iter().filter_map(|p| dom.get(p)).collect())
                .unwrap_or_default();

            let mut new_dom: HashSet<String> = all_labels.clone();
            for pd in &pred_doms {
                new_dom = new_dom.intersection(pd).cloned().collect();
            }
            new_dom.insert(label.clone());

            if dom.get(label) != Some(&new_dom) {
                dom.insert(label.clone(), new_dom);
                changed = true;
            }
        }
    }

    dom
}

/// Immediate dominator: for each block, find the strict dominator that doesn't
/// dominate any other dominator of the block.
pub fn immediate_dominators(cfg: &CFG) -> HashMap<String, Option<String>> {
    let dom = dominator_tree(cfg);
    let mut idom: HashMap<String, Option<String>> = HashMap::new();

    for block in &cfg.blocks {
        let label = &block.label;
        if label == &cfg.entry {
            idom.insert(label.clone(), None);
            continue;
        }

        let dominators = dom.get(label).cloned().unwrap_or_default();
        // Strict dominators: all dominators except self
        let strict_doms: HashSet<&String> = dominators.iter().filter(|d| *d != label).collect();

        // Immediate dominator is the strict dominator that is dominated by all other strict dominators
        let mut best: Option<&String> = None;
        for candidate in &strict_doms {
            let candidate_doms = dom.get(*candidate);
            let is_idom = strict_doms.iter().all(|other| {
                if other == candidate {
                    return true;
                }
                // candidate should be dominated by other
                candidate_doms.map_or(false, |cd| cd.contains(*other))
            });
            if is_idom {
                best = Some(candidate);
                break;
            }
        }

        idom.insert(label.clone(), best.cloned());
    }

    idom
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ternary;

    #[test]
    fn test_cfg_simple() {
        let ops = vec![Op::Push(Ternary::Pos), Op::Halt];
        let cfg = CFG::from_ops(&ops);
        assert!(cfg.block_count() >= 1);
        assert_eq!(cfg.entry, "bb0");
    }

    #[test]
    fn test_cfg_branch_creates_blocks() {
        let ops = vec![
            Op::Push(Ternary::Pos),
            Op::Branch("pos_path".to_string(), "neg_path".to_string()),
            Op::Push(Ternary::Zero),
            Op::Merge,
            Op::Push(Ternary::Neg),
            Op::Merge,
            Op::Halt,
        ];
        let cfg = CFG::from_ops(&ops);
        assert!(cfg.block_count() > 1);
    }

    #[test]
    fn test_cfg_block_successors() {
        let ops = vec![
            Op::Push(Ternary::Pos),
            Op::Branch("pos".to_string(), "neg".to_string()),
            Op::Halt,
        ];
        let cfg = CFG::from_ops(&ops);
        let first = cfg.get_block("bb0").unwrap();
        assert!(first.successors.contains(&"pos".to_string()));
        assert!(first.successors.contains(&"neg".to_string()));
    }

    #[test]
    fn test_dominator_tree_simple() {
        let ops = vec![Op::Push(Ternary::Pos), Op::Halt];
        let cfg = CFG::from_ops(&ops);
        let dom = dominator_tree(&cfg);
        // Entry block dominates itself
        let entry_doms = dom.get("bb0").unwrap();
        assert!(entry_doms.contains("bb0"));
    }

    #[test]
    fn test_dominator_tree_entry() {
        let ops = vec![
            Op::Push(Ternary::Pos),
            Op::Branch("a".to_string(), "b".to_string()),
            Op::Halt,
        ];
        let cfg = CFG::from_ops(&ops);
        let dom = dominator_tree(&cfg);
        // Entry should dominate everything
        let entry_doms = dom.get("bb0").unwrap();
        assert!(entry_doms.contains("bb0"));
    }

    #[test]
    fn test_immediate_dominators() {
        let ops = vec![Op::Push(Ternary::Pos), Op::Halt];
        let cfg = CFG::from_ops(&ops);
        let idom = immediate_dominators(&cfg);
        // Entry has no immediate dominator
        assert_eq!(idom.get("bb0"), Some(&None));
    }

    #[test]
    fn test_cfg_empty() {
        let ops: Vec<Op> = vec![];
        let cfg = CFG::from_ops(&ops);
        assert_eq!(cfg.block_count(), 0);
        assert_eq!(cfg.entry, "bb0");
    }

    #[test]
    fn test_cfg_loop_structure() {
        // Simulate a simple loop: branch back to entry
        let ops = vec![
            Op::Push(Ternary::Pos),
            Op::Branch("loop_back".to_string(), "exit".to_string()),
            Op::Push(Ternary::Zero),
            Op::Halt,
        ];
        let cfg = CFG::from_ops(&ops);
        assert!(cfg.block_count() >= 2);
    }
}
