# Ternary Compiler — Balanced-Ternary Logic Backend

**Ternary Compiler** is a compiler backend for ternary logic expressions, lowering source-level ternary operations into a sequence of native ternary instructions. It defines the fundamental `Trit` type (False/Unknown/True mapped to -1/0/+1) and emits instruction sequences using opcodes tailored to three-valued logic: AND, OR, NOT, Consensus, Load, Store, and Halt.

## Why It Matters

Three-valued logic (tri-state logic) is foundational to hardware design (Verilog `z`/`x` states), database NULL semantics (SQL three-valued logic), and fuzzy decision systems. A compiler that targets ternary instructions natively — rather than encoding ternary operations as pairs of binary operations — enables direct execution on ternary-aware hardware and produces more compact code. The instruction set's `Consensus` opcode directly implements the Kleene consensus operator, which is essential for agreement protocols in distributed ternary agent systems.

## How It Works

### Trit Encoding

Each trit is represented as a signed 8-bit integer with three valid states: -1 (False), 0 (Unknown), 1 (True). The `from_i8` constructor validates range and rejects values outside {-1, 0, +1}. This encoding is compatible with standard integer arithmetic while enforcing ternary constraints at the type level.

### Instruction Set

The compiler emits seven opcodes:

- **AND / OR** — Kleene three-valued logic: `AND(Unknown, False) = False`, `OR(Unknown, True) = True`
- **NOT** — Logical negation: swaps True ↔ False, Unknown stays Unknown
- **Consensus** — Returns the agreement value if both operands agree, Unknown otherwise
- **Load / Store** — Memory operations for ternary operands
- **Halt** — Terminates execution

### Compilation

The `compile(source)` function parses source text and produces a `CompilerResult` containing the instruction vector and a symbol table mapping identifiers to memory slots. Compilation is O(n) in the source length for linear pass, producing O(n) instructions.

### Kleene Logic Truth Tables

```
AND:  F U T      OR:   F U T      NOT:
    F F F F          F   F U T      F → T
    U F U U          U   U U T      U → U
    T F U T          T   T T T      T → F
```

## Quick Start

```rust
use ternary_compiler::{Trit, OpCode, Instruction, compile};

// Trit operations
let t = Trit::True;
assert_eq!(t.to_i8(), 1);
assert_eq!(Trit::from_i8(-1), Some(Trit::False));

// Compile source to ternary instructions
let result = compile("consensus r0 r1")?;
println!("Generated {} instructions", result.instructions.len());
```

```bash
cargo add ternary-compiler
```

## API

| Type / Function | Description |
|---|---|
| `Trit` | Three-valued logic type: `False(-1)`, `Unknown(0)`, `True(1)` |
| `OpCode` | Instruction opcodes: And, Or, Not, Consensus, Load, Store, Halt |
| `Instruction` | Opcode + operand vector |
| `CompilerResult` | Instructions + symbol table |
| `compile(&str)` | Parse source → `CompilerResult` |

## Architecture Notes

This is the original ternary logic backend in the **SuperInstance** ecosystem. It provides the instruction-set architecture (ISA) that ternary-compiler-v2 extends with full IR and register allocation. The `Consensus` opcode implements the core agreement primitive used by ternary-consensus for Byzantine-tolerant distributed voting, where γ (growth signal) + η (entropy) = C (constant conservation). See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Kleene, Stephen C. *Introduction to Metamathematics*, 1952 — three-valued logic (Kleene K₃).
- Knuth, Donald E. *The Art of Computer Programming, Vol. 2*, §4.1 — balanced ternary notation.
- Cohn, P. M. *Universal Algebra*, 1965 — algebraic structures on three-element lattices.

## License

MIT
