# ternary-compiler

Compiles ternary strategy descriptions into optimized lookup tables — the **compiler** for the ternary runtime.

## Compilation Pipeline

```text
Strategy text ──parse──▶ StrategyIR ──optimize──▶ ──compile──▶ CompiledPolicy
                                                           ↕
                                                 Profiler / Disassembler
```

### Stage 1: StrategyIR

`StrategyIR` is the intermediate representation. A strategy is a sequence of **trits** (ternary digits: `-`, `0`, `+`) with per-position metadata:

- **Trit value**: Negative (-1), Zero (0), or Positive (+1)
- **Stability flag**: whether the position is known-stable
- **Label**: optional human-readable name

```rust
use ternary_compiler::{StrategyIR, Trit};

let ir = StrategyIR::parse("my-strategy", "-0+0-+");
```

### Stage 2: Optimizer

The optimizer runs passes on the action table:

- **Dead-code elimination**: Stable neutral (zero) positions carry no information → `Eliminated`
- **Constant folding**: A neutral position surrounded by identical non-neutral actions is folded to match its neighbors

```rust
use ternary_compiler::Optimizer;

let mut actions = vec![Action::Neutral, Action::Commit, Action::Neutral];
let stable = vec![true, false, false];
Optimizer::new().optimize(&mut actions, &stable);
// actions[0] → Eliminated (dead code: stable + neutral)
```

### Stage 3: CompiledPolicy

The `CompiledPolicy` is an optimized lookup table: **index → action in O(1)**.

```rust
use ternary_compiler::Compiler;

let policy = Compiler::new().compile(&ir);
let action = policy.action(2); // O(1) lookup
```

### Stage 4: Profiler

Profile a compiled policy against environments to identify hot/cold paths:

```rust
use ternary_compiler::{Profiler, Trit};

let mut profiler = Profiler::new(&policy);
profiler.evaluate(&policy, &vec![Trit::Positive, Trit::Negative, Trit::Zero]);
let report = profiler.report(&policy);
println!("Hottest: {:?}", report.hottest_path());
```

### Stage 5: Disassembler

Convert a `CompiledPolicy` back to human-readable form:

```rust
use ternary_compiler::Disassembler;

let text = Disassembler::to_text(&policy);     // "+-0x"
let detailed = Disassembler::to_detailed(&policy); // multi-line report
let ir = Disassembler::to_ir(&policy);          // back to StrategyIR
```

## Design

- **Pure Rust**, no unsafe code, no external dependencies
- O(1) compiled lookup tables
- Configurable optimization passes
- Round-trippable: compile → disassemble → re-compile

## License

MIT

## See Also
- **ternary-compiler-v2** — related
- **ternary-compiler-optimizer** — related
- **ternary-grammar** — related
- **ternary-language** — related
- **ternary-logic** — related

