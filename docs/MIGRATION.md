## Migrating from Binary

If you're used to binary compilers (valid/invalid AST), ternary compilation adds a **warning** state — the $0$ state where the code is valid but suboptimal.

| Binary | Ternary |
|--------|---------|
| Valid ($1$) | Optimal ($+1$) |
| Invalid ($0$) | Warning ($0$) |
| | Error ($-1$) |

Binary compilation forces all valid code into one bucket. Ternary lets the compiler distinguish "this is correct" from "this is correct but could be better." The warning state carries information that binary compilation discards.

See **[From Binary to Ternary](https://github.com/SuperInstance/ternary-cookbook/blob/master/guides/FROM_BINARY.md)** for the full migration guide.
