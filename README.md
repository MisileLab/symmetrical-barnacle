# Flux Language Compiler

Flux is an experimental functional programming language with:

- **Zero-allocation safety** by default
- **Effect system** for tracking side effects, allocation, and execution location
- **Haskell-style syntax** with type inference
- **Arena-based memory management**
- **GPU-native concepts** with CPU/GPU execution tracking
- **Parallel primitives** (par_for, par_map, etc.)
- **Actor model** for concurrent state management
- **Native compilation** to x86-64 and WebAssembly

## Features

### Effect System

Every function has an effect set that tracks:

- **Purity**: `pure`, `io`, `state`, `debug`
- **Execution space**: `cpu`, `gpu`
- **Allocation**: `alloc none`, `alloc arena`, `alloc heap`
- **Concurrency**: `single`, `concurrent`

Default effects are `!{pure, cpu, alloc none}` if not specified.

### Zero Allocation by Default

Functions with `alloc none` cannot:
- Allocate heap memory
- Create new arenas
- Call functions that allocate

This is enforced at compile time!

### Example Programs

See `examples/` directory for sample Flux programs.

## Building

```bash
cd flux_lang
cargo build --release
```

## Usage

### Type check a file

```bash
cargo run --bin fluxc -- check examples/basic.flux
```

### Compile to native x86-64

```bash
cargo run --bin fluxc -- build examples/basic.flux --target x86_64
```

### Compile to WebAssembly

```bash
cargo run --bin fluxc -- build examples/basic.flux --target wasm32
```

### Build and run (x86-64 only)

```bash
cargo run --bin fluxc -- run examples/basic.flux
```

## Project Structure

```
flux_lang/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── ast.rs            # Abstract syntax tree
│   ├── lexer.rs          # Lexical analyzer
│   ├── parser.rs         # Parser
│   ├── types.rs          # Type system
│   ├── effect.rs         # Effect system
│   ├── typecheck.rs      # Type & effect checker
│   ├── codegen.rs        # IR code generator
│   ├── backend_x86.rs    # x86-64 AOT compiler
│   ├── backend_wasm.rs   # WebAssembly compiler
│   └── runtime.rs        # Runtime primitives
├── examples/             # Example Flux programs
└── tests/                # Integration tests
```

## Running Tests

```bash
cargo test
```

## Language Syntax

### Function Definition

```flux
-- Function with explicit effects
add: i32 -> i32 -> i32 !{pure, cpu, alloc none}
add x y = x + y

-- Function with default effects
inc: i32 -> i32
inc x = x + 1
```

### Let Bindings

```flux
main: i32
main = let x = 1 in
       let y = 2 in
       x + y
```

### Conditionals

```flux
abs: i32 -> i32
abs x = if x < 0 then -x else x
```

### Effect Violations

This will fail to compile:

```flux
foo: i32 -> i32 !{pure, cpu, alloc heap}
foo x = x + 1

bar: i32 -> i32 !{pure, cpu, alloc none}
bar x = foo x  -- ERROR: alloc none cannot call alloc heap
```

## Implementation Notes

- Uses Cranelift for x86-64 code generation
- Generates real ELF/Mach-O object files, not JIT
- Links with system C compiler (gcc/clang)
- WebAssembly output via wasm-encoder
- Full effect checking at compile time
- No garbage collection - arena-based memory model

## License

MIT
