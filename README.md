# ternary-compiler

**Parse, compile, optimize, and profile ternary logic expressions. A full compiler pipeline from strategy IR to optimized bytecode.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Background

Ternary logic — three-valued logic with states {−1, 0, +1} — generalizes Boolean logic with a third "unknown" or "neutral" state. Introduced independently by Łukasiewicz (1920) and Kleene (1938), it appears naturally in database NULL semantics, hardware don't-care states, and multi-agent voting systems.

`ternary-compiler` provides a complete compilation pipeline for ternary expressions and strategies:

1. **Expression-level**: An AST with `And`, `Or`, `Not`, `Min`, `Max`, `If` nodes, evaluated via tree-walking or compiled to a stack-machine bytecode.
2. **Strategy-level**: A `StrategyIR` representation of named ternary vectors, compiled through optimization passes into `CompiledPolicy` lookup tables.
3. **Profiler**: Statistical analysis of which policy positions are hot/cold under various environments.

## How It Works

### Expression Compiler

The `Expr` AST supports:
- `Const(TV)`, `Var(String)` — Literals and variables
- `Not(e)` — Negation (sign flip)
- `And(a, b)`, `Or(a, b)` — Min/max semantics (Łukasiewicz conjunction/disjunction)
- `Min(a, b)`, `Max(a, b)` — Direct min/max
- `If(cond, then, else)` — Conditional (positive guard)

**Constant folding** collapses expressions like `And(Const(+1), Const(0))` → `Const(0)` at compile time.

**Bytecode compilation** produces a stack-machine program:
- `PushConst(i8)`, `Load(slot)`, `Neg`, `Min`, `Max`, `JumpIfNotPlus(addr)`
- `Compiler::execute()` runs bytecode against a slot array.

### Strategy IR → Compiled Policy

The compilation pipeline:

```
StrategyIR (trits + metadata + stability flags)
    ↓ Compiler::compile_raw()
    ↓ Optimizer (dead-code elimination + constant folding)
    ↓
CompiledPolicy (O(1) lookup table)
```

- **`Trit`** — Canonical balanced-ternary digit with `from_char`/`as_char` for text I/O (`-`, `0`, `+`).
- **`StrategyIR`** — Named sequence of trits with per-position stability flags and labels. Parses from text strings like `"-0+0-+"`.
- **`Optimizer`** — Two passes: dead-code elimination (stable + neutral → eliminated) and constant folding (neutral surrounded by identical actions → fold).
- **`CompiledPolicy`** — Optimized lookup table with O(1) index → action lookup. Tracks elimination count and compression ratio.
- **`Action`** — Commit (+1), Oppose (−1), Neutral (0), Eliminated (optimized away).

### Profiler

Evaluates a `CompiledPolicy` against environments (trit vectors) and reports:
- Per-position hit counts (hot/cold paths)
- Hottest and coldest active positions
- Total evaluation counts

### Disassembler

Converts `CompiledPolicy` back to:
- **Text**: `"+-0x"` (compact)
- **Detailed report**: Multi-line with indices, actions, trits
- **StrategyIR**: Lossy reverse (eliminated → zero)

## Experimental Results

The test suite verifies:
- **Expression evaluation**: `Not(+1) = −1`, `And(+1, −1) = −1`, `Or(+1, −1) = +1`.
- **Constant folding**: `And(Const(+1), Const(0))` → `Const(0)`.
- **If-semantics**: `If(+1, a, b) = a`, `If(0, a, b) = b`.
- **Bytecode execution**: `Max(Const(−1), Const(+1))` compiles and executes to `+1`.
- **Optimizer passes**: Dead-code eliminates stable neutrals; constant folding collapses surrounded neutrals.
- **Profiler**: Correctly identifies hot paths (position 0 with 10 hits) and cold paths.
- **Disassembler**: Text round-trip preserves actions; detailed output includes compression ratio.

## Impact

This crate is the *compiler infrastructure* of the ternary fleet. It transforms high-level strategy descriptions into optimized, O(1)-lookup decision tables. The profiling subsystem enables data-driven optimization: identify which strategy positions matter under real workloads and eliminate the rest.

## Use Cases

1. **Strategy Compilation** — Define a ternary strategy as a text string (`"-0+0-+0"`), compile it through dead-code elimination and constant folding, and get an optimized lookup table for real-time evaluation.
2. **Ternary Logic Circuits** — Use the expression compiler to define, optimize, and execute three-valued logic circuits. Constant folding removes redundant gates at compile time.
3. **Policy Profiling** — Feed real-world environments through the profiler to identify which policy positions are critical and which can be eliminated, reducing memory and compute.
4. **Reverse Engineering** — The disassembler converts compiled policies back to human-readable form for auditing and debugging.

## Open Questions

1. **Register allocation** — The current bytecode uses a simple hash-based slot assignment. A proper register allocator would reduce stack pressure and enable better optimization.
2. **Conditional compilation** — The `If` node currently compiles to a simplified form. Full conditional jumps with back-patching would enable arbitrary ternary control flow.
3. **Multi-strategy linking** — Can multiple `CompiledPolicy` instances be linked together for hierarchical strategy evaluation?

## Connection to Oxide Stack

`ternary-compiler` sits at the center of the ternary fleet, bridging high-level strategy definitions (from `ternary-core` types) with low-level execution (feeding into `ternary-compiler-optimizer` for further optimization). The `StrategyIR` format serves as the interchange between strategy authoring tools and the runtime. The profiler and disassembler form the observability layer, enabling users to understand what their strategies are doing and why.
