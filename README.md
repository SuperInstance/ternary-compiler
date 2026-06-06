# ternary-compiler

A compiler that converts ternary decisions `{-1, 0, +1}` into simple bytecode for coordination. Takes ternary-mud room algebra and emits executable coordination instructions.

## Overview

This library implements a full compilation pipeline for ternary expressions:

```
Source → Lexer → Tokens → Parser → AST → Compiler → Bytecode → Optimizer → VM Execution
```

### Ternary Arithmetic

Ternary values follow balanced ternary arithmetic with wrapping:

| Operation | Result |
|-----------|--------|
| (-1) + (-1) | +1 (wraps) |
| (+1) + (+1) | -1 (wraps) |
| (-1) + (+1) | 0 |
| (-1) × (+1) | -1 |
| 0 × anything | 0 |
| !(+1) | -1 |
| !0 | 0 |
| !(-1) | +1 |

## Architecture

### Modules

1. **`lexer`** — Tokenize ternary expressions into tokens: `NEGATIVE`, `ZERO`, `POSITIVE`, `ROOM`, `PASSAGE`, `GATE`, `SEQUENCE`, `PARALLEL`, `BRANCH`
2. **`ast`** — Abstract syntax tree: `TernaryExpr`, `RoomDef`, `PassageDef`, `GateDef`, `Block`
3. **`compiler`** — Compile AST to bytecode operations
4. **`vm`** — Simple stack-based VM that executes bytecode with room context tracking
5. **`optimizer`** — Constant folding, dead code elimination, room merge
6. **`ir`** — Intermediate representation: `BasicBlock`, `ControlFlowGraph`, dominator tree

### Core Types

```rust
// Core ternary value
enum Ternary { Neg = -1, Zero = 0, Pos = 1 }

// Bytecode operations
enum Op {
    Push(Ternary), Add, Mul, Negate,
    EnterRoom(String), LeaveRoom,
    Branch(String, String), Merge, Halt,
}

// Compiled bytecode program
struct Bytecode { ops: Vec<Op>, constants: Vec<Ternary>, rooms: Vec<String> }

// Stack-based virtual machine
struct VM { stack: Vec<Ternary>, room: Option<String>, pc: usize }

// IR basic block
struct BasicBlock { label: String, ops: Vec<Op>, successors: Vec<String> }

// Control flow graph
struct CFG { blocks: Vec<BasicBlock>, entry: String }
```

## Usage

```rust
use ternary_compiler::{Ternary, Op, VM};
use ternary_compiler::lexer::Lexer;
use ternary_compiler::ast::Parser;
use ternary_compiler::compiler::Compiler;
use ternary_compiler::optimizer::Optimizer;

// Full pipeline: source → execution
let source = "neg + neg";  // (-1) + (-1) = +1
let mut lexer = Lexer::new(source);
let tokens = lexer.tokenize().unwrap();
let mut parser = Parser::new(tokens);
let ast = parser.parse().unwrap();
let mut compiler = Compiler::new();
let result = compiler.compile(&ast);

// Optimize bytecode
let optimized = Optimizer::optimize(result.bytecode.ops);

// Execute
let mut vm = VM::new();
let value = vm.run(&optimized);
```

## Syntax

| Syntax | Meaning |
|--------|---------|
| `neg` / `zero` / `pos` | Ternary literals (-1, 0, +1) |
| `a + b` | Ternary addition (wrapping) |
| `a * b` | Ternary multiplication |
| `!a` | Ternary negation |
| `a >> b` | Sequence (do a then b) |
| `a || b` | Parallel (do both, merge) |
| `room name { body }` | Room definition |
| `passage name from -> to` | Passage between rooms |
| `gate name (cond, neg, zero, pos)` | Conditional gate |

## Building

```bash
cargo build
cargo test
```

## License

MIT
