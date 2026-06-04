# Future Integration: ternary-compiler

## Current State

ternary-compiler compiles ternary strategy descriptions into optimized lookup tables. The pipeline: `Strategy text → StrategyIR → Optimizer → Compiler → CompiledPolicy`. `StrategyIR` represents the strategy with `StrategyMetadata` and `PositionInfo`. `CompiledPolicy` contains `Action` entries for fast execution. `Optimizer` improves strategy performance. `Profiler` generates `ProfileReport` with `PathStats` for hot-path analysis. `Disassembler` reverse-compiles policies for debugging. Constants: `MAX_POSITIONS = 64`, `MAX_ENVIRONMENTS = 256`.

## Integration Opportunities

### ESP32/WASM/DGX Compilation Targets (Primary Integration)

ternary-compiler is the bridge between high-level strategy and bare-metal deployment:

- **ESP32 target**: `CompiledPolicy` compiles to `ternary-esp32-firmware`'s 279-byte lookup table. `MAX_POSITIONS = 64` maps to 4-trit inputs (3^4 = 81 entries, 81 × ~3.4 bytes ≈ 279 bytes). The `Optimizer` removes unused paths, `Profiler` identifies which entries are actually accessed, and the compiler emits the minimal binary.
- **WASM target**: `CompiledPolicy` compiles to `ternary-wasm` bytecode for browser-based BrowserRoom. The formula is the same, but the output format changes: WASM bytecode instead of C arrays.
- **DGX target**: `CompiledPolicy` compiles to CUDA kernels. The `Profiler`'s `PathStats` identify parallelizable paths. `MAX_ENVIRONMENTS = 256` environments can be evaluated simultaneously on 3,072 CUDA cores.

### Training → Compilation → Deployment Pipeline

The full pipeline from ROOM-AS-CODESPACE-ARCHITECTURE.md:

1. **Train on DGX**: Evolve strategies using `ternary-evolution` with GPU acceleration. `Profiler` tracks which strategies perform best.
2. **Optimize on Pi**: `Optimizer` reduces strategy complexity. `MAX_POSITIONS = 64` ensures it fits on constrained hardware.
3. **Compile for ESP32**: `Compiler` emits `CompiledPolicy` → `ternary-esp32-firmware` (279 bytes, 8ns lookup).
4. **Verify**: `Disassembler` reverse-compiles and `Profiler` validates performance matches training.

### ternary-cell → Grid Strategy Compilation

A converged `CellGrid` (from ternary-cell) can be compiled to a `CompiledPolicy`. Each cell's `consensus()` value at each grid position maps to a `PositionInfo` entry in `StrategyIR`. The tissue-level `tissue_balance()` becomes the strategy's metadata. Pipeline: `CellGrid → StrategyIR → Compiler → CompiledPolicy → ESP32 firmware`.

### agentic-compiler → Runtime Self-Optimization

The dormant `agentic-compiler` crate's profiling and JIT capabilities map to ternary-compiler's `Profiler`:

1. `Profiler` identifies hot paths in the `CompiledPolicy`
2. `Optimizer` rewrites those paths for better performance
3. `Compiler` emits an updated `CompiledPolicy`
4. Deploy the updated policy without restarting the agent

This is construct-core's `SELF_IMPROVEMENT` skill: the agent profiles itself, optimizes, and recompiles.

## Potential in Mature Systems

ternary-compiler becomes the deployment pipeline's linchpin. Every strategy, whether trained on a DGX or hand-crafted, goes through the compiler before deployment. The `Profiler` ensures optimal performance. The `Optimizer` reduces resource usage. The `Disassembler` enables debugging on deployed hardware. Multiple compilation targets mean the same strategy runs everywhere: ESP32 (279 bytes), WASM (browser), CUDA (GPU), native binary (workstation).

## Cross-Pollination Ideas

- **CompiledPolicy → ternary-protocol wire format**: Compile policies into ternary-protocol messages for over-the-air strategy updates. Flash new strategies to ESP32 without firmware update.
- **Profiler → fleet-wide optimization**: Aggregate `ProfileReport` from all fleet agents. Identify globally common hot paths. Compile optimized shared policies.
- **Disassembler → PLATO tile**: Reverse-compiled policies become human-readable tiles: "Strategy X avoids positions 3,7,12 and chooses position 5."

## Dependencies for Next Steps

1. ESP32 firmware emission backend (279-byte format)
2. WASM bytecode emission backend
3. CUDA kernel emission backend
4. `Profiler` → `Optimizer` → `Compiler` closed-loop (self-optimization)
5. `CompiledPolicy` → `ternary-protocol` over-the-air update format
