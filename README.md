# ternary-compiler

A complete compiler backend for ternary logic programs. Implements a six-stage pipeline: **Lexer → Parser (AST) → Compiler (Bytecode) → Optimizer → IR (CFG + Dominator Tree) → VM**. The source language expresses ternary computations using `neg`, `zero`, `pos` literals, arithmetic/logical operators, rooms, passages, gates, and branching.

## Why It Matters

Most ternary logic work stops at the gate level. This crate provides the full **source-to-execution** toolchain: you write programs in a human-readable ternary language, compile them to stack-based bytecode, optimize them, analyze their control flow, and execute them on a virtual machine — all with three-valued semantics.

This is the foundation for **agent scripting in uncertain environments**: programs that reason about truth values that may be unknown (0) rather than forced into binary true/false.

Within the **γ + η = C** framework:

| Symbol | Domain |
|--------|--------|
| γ | `Trit` ∈ {Neg(−1), Zero(0), Pos(+1)} — compiled values |
| η | Optimization decisions: fold, eliminate, merge |
| C | Semantic constraints: ternary arithmetic (mod-3 addition), control-flow integrity |

## How It Works

### Pipeline Overview

```
  Source: "room start { pos + neg } >> gate check(pos, neg, zero, pos)"
      │
      ▼  Lexer
  Tokens: [Room("start"), LBrace, Positive, Plus, Negative, RBrace,
           Sequence, Gate("check"), LParen, Positive, Comma, ...]
      │
      ▼  Parser (recursive descent)
  AST:   Sequence(Room("start", Add(Lit(Pos), Lit(Neg))), Gate(...))
      │
      ▼  Compiler
  Bytecode: [EnterRoom("start"), Push(Pos), Push(Neg), Add, LeaveRoom,
             Push(Pos), Branch(...), ..., Halt]
      │
      ▼  Optimizer
  Optimized: [EnterRoom("start"), Push(Zero), LeaveRoom, ..., Halt]
      │           (constant-folded: Pos + Neg = Zero)
      ▼  IR (CFG)
  Basic Blocks + Dominator Tree
      │
      ▼  VM
  Result: Trit (top of stack at Halt)
```

### Ternary Arithmetic

The VM implements **balanced ternary arithmetic** with modular wrapping:

$$a + b = \begin{cases} \text{Pos} & \text{if } a + b \equiv +1 \pmod{3}^* \\ \text{Zero} & \text{if } a + b = 0 \\ \text{Neg} & \text{if } a + b \equiv -1 \pmod{3}^* \end{cases}$$

Concrete addition table:

| + | Neg | Zero | Pos |
|---|-----|------|-----|
| **Neg** | Pos | Neg | Zero |
| **Zero** | Neg | Zero | Pos |
| **Pos** | Zero | Pos | Neg |

Multiplication follows sign rules: `Neg × Neg = Pos`, `Pos × Neg = Neg`, anything × Zero = Zero.

Negation: `Neg ↔ Pos`, `Zero → Zero`.

### Lexer

The lexer tokenizes the input character-by-character with one-character lookahead:

| Token | Lexeme |
|-------|--------|
| `Negative` | `neg`, `Neg`, `NEG` |
| `Zero` | `zero`, `Zero`, `ZERO`, `0` |
| `Positive` | `pos`, `Pos`, `POS` |
| `Sequence` | `>>` |
| `Parallel` | `\|\|` |
| `Branch` | `?` |
| `Arrow` | `->` |
| `Room(name)` | `room <ident>` |
| `Passage(name)` | `passage <ident>` |
| `Gate(name)` | `gate <ident>` |
| `LParen/RParen` | `( )` |
| `LBrace/RBrace` | `{ }` |
| `Plus/Minus/Star` | `+ - *` |
| `Bang` | `!` |
| `Semicolon/Comma` | `; ,` |
| `Ident` | alphanumeric identifier |
| `Eof` | end of input |

**Complexity**: O(n) where n = source length.

### Parser (Recursive Descent)

Precedence (lowest to highest):

1. **Sequence** (`>>`) — left associative
2. **Parallel** (`||`) — left associative
3. **Additive** (`+`, `-`) — left associative
4. **Multiplicative** (`*`) — left associative
5. **Unary** (`!`) — prefix
6. **Primary** (literals, variables, parenthesized expressions, rooms, gates)

Grammar (simplified EBNF):

```
expr     := sequence
sequence := parallel ( ">>" parallel )*
parallel := additive ( "||" additive )*
additive := mul ( ( "+" | "-" ) mul )*
mul      := unary ( "*" unary )*
unary    := "!" unary | primary
primary  := "neg" | "zero" | "pos" | ident
          | "(" expr ")"
          | "room" ident "{" expr "}"
          | "passage" ident ident "->" ident
          | "gate" ident "(" expr "," expr "," expr "," expr ")"
```

**Complexity**: O(n) with recursive descent; one token consumed per recursive call.

### Compiler (AST → Bytecode)

The compiler walks the AST and emits stack-based bytecode:

| OpCode | Stack Effect | Description |
|--------|-------------|-------------|
| `Push(Trit)` | `[] → [t]` | Push literal |
| `Add` | `[a, b] → [a+b]` | Ternary addition |
| `Mul` | `[a, b] → [a×b]` | Ternary multiplication |
| `Negate` | `[a] → [¬a]` | Ternary negation |
| `Branch(pos, neg)` | `[c] → ...` | Three-way branch |
| `Merge` | `[] → []` | Branch path merge point |
| `EnterRoom(name)` | `[] → []` | Set room context |
| `LeaveRoom` | `[] → []` | Clear room context |
| `Halt` | `[] → []` | Terminate execution |

Gates compile to: `condition → Branch → [pos_path, Merge] → [zero_path, Merge] → [neg_path, Merge]`.

### Optimizer

Three passes:

1. **Constant Folding**: `Push(a), Push(b), Add` → `Push(a+b)`; `Push(a), Negate` → `Push(¬a)`.
2. **Dead Code Elimination**: Remove all instructions after `Halt`.
3. **Room Merging**: Remove `LeaveRoom` immediately followed by `EnterRoom(same)`.

Each pass is O(n) in bytecode length. The full pipeline runs in O(n).

### IR: Control Flow Graph

The CFG partitions bytecode at branch points, merge points, and halt instructions into **basic blocks** — maximal straight-line sequences with no internal branches.

```
bb0: Push(Pos), Branch("pos_path", "neg_path")
      │ successors: ["pos_path", "neg_path"]
bb1: Push(Zero), Merge
      │ successors: [bb2]
bb2: Push(Neg), Halt
      │ successors: []
```

### Dominator Tree

The dominator tree is computed using the **iterative dataflow algorithm**:

$$D(n) = \{n\} \cup \bigcap_{p \in \text{preds}(n)} D(p)$$

Starting with $D(\text{entry}) = \{\text{entry}\}$ and $D(n) = V$ (all nodes) for all others, iterate until convergence. The immediate dominator of *n* is the strict dominator of *n* that is dominated by all other strict dominators of *n*.

**Complexity**: O(n² · d) worst-case where d = CFG depth, though in practice convergence is fast (2–5 iterations for typical programs).

### VM Execution

The VM is a simple stack machine:

```
loop {
    match ops[pc] {
        Push(t)    => stack.push(t); pc += 1,
        Add        => { b = pop(); a = pop(); push(a + b); pc += 1 },
        Mul        => { b = pop(); a = pop(); push(a × b); pc += 1 },
        Negate     => { a = pop(); push(¬a); pc += 1 },
        Branch     => { c = pop(); jump to pos/zero/neg path },
        Merge      => pc += 1,  // nop
        EnterRoom  => room = Some(name); pc += 1,
        LeaveRoom  => room = None; pc += 1,
        Halt       => return Halted(stack.top()),
    }
}
```

**Complexity**: O(n) for a single execution pass (n = bytecode length).

## Quick Start

```rust
use ternary_compiler::lexer::Lexer;
use ternary_compiler::ast::Parser;
use ternary_compiler::compiler::Compiler;
use ternary_compiler::optimizer::Optimizer;
use ternary_compiler::vm::{VM, VMResult};
use ternary_compiler::Ternary;

// Lex
let mut lexer = Lexer::new("neg + neg");
let tokens = lexer.tokenize().unwrap();

// Parse
let mut parser = Parser::new(tokens);
let ast = parser.parse().unwrap();

// Compile
let mut compiler = Compiler::new();
let result = compiler.compile(&ast);

// Optimize
let optimized = Optimizer::optimize(result.bytecode.ops.clone());

// Execute
let mut vm = VM::new();
let value = vm.run(&optimized);
// Neg + Neg = Pos (ternary wrapping addition)
assert_eq!(value, VMResult::Halted(Ternary::Pos));
```

## API

| Module | Key Types |
|--------|-----------|
| `lexer` | `Lexer`, `Token`, `LexError` |
| `ast` | `Parser`, `TernaryExpr`, `RoomDef`, `GateDef`, `PassageDef`, `ParseError` |
| `compiler` | `Compiler`, `CompileResult` |
| `optimizer` | `Optimizer` (`constant_fold`, `dead_code_elimination`, `merge_rooms`, `optimize`) |
| `ir` | `BasicBlock`, `CFG`, `dominator_tree`, `immediate_dominators` |
| `vm` | `VM`, `VMResult` |
| (root) | `Trit`, `Op`, `OpCode`, `Bytecode`, `CompilerResult` |

## Architecture Notes

The compiler is designed as a **reference implementation** for ternary language semantics. Each stage is a clean module boundary so that individual components (lexer, parser, optimizer, VM) can be replaced or reused independently.

The `Gate` construct is the ternary generalization of `if/else`: it has three branches (positive, zero, negative) rather than two, matching the three-valued logic model. This is why the IR needs `Merge` opcodes — each gate produces three execution paths that must converge.

The VM's `Branch` implementation scans forward for `Merge` opcodes to find path boundaries. A production compiler would use proper labels or computed offsets, but the scan approach keeps the bytecode format simple for debugging and introspection.

## References

- **Aho, A. V., Lam, M. S., Sethi, R., & Ullman, J. D.** (2006). *Compilers: Principles, Techniques, and Tools* (2nd ed.). — Dragon book: lexing, parsing, IR construction, dominator trees.
- **Cooper, K. D., & Torczon, L.** (2011). *Engineering a Compiler* (2nd ed.). — Practical compiler construction, constant folding, dead code elimination.
- **Allen, F. E.** (1970). "Control Flow Analysis." *ACM SIGPLAN Notices*, 5(7), 1–19. — Foundations of CFG and dominator analysis.
- **Knuth, D. E.** (1962). "Optimum Binary Search Trees." *Acta Informatica*, 1, 14–25. — Relevant to ternary search/branch structures.
- **Avizienis, A.** (1961). "Signed-Digit Number Representations for Fast Parallel Arithmetic." *IRE Transactions on Electronic Computers*, EC-10, 389–400. — Balanced ternary arithmetic.

## License

MIT
