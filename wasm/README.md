# WASM build support

JayJay compiles its Rust core and Tree-sitter grammars into one `wasm32-unknown-unknown` module. The C header overlay in this directory is needed only for that cross-compilation; native macOS and Linux builds do not use it.

## Why the overlay exists

LLVM provides the WebAssembly code generator and compiler built-in headers, but `wasm32-unknown-unknown` has no corresponding C libc or platform sysroot. Host headers cannot be reused: macOS headers describe Darwin, and Linux headers describe the host's libc and ABI.

Tree-sitter Language 0.1.7 supplies a minimal target-specific libc shim through `DEP_TREE_SITTER_LANGUAGE_WASM_HEADERS`. JayJay's complete grammar set uses APIs missing from that published shim, so `scripts/llvm-clang` searches headers in this order:

1. `wasm/sysroot/include`, containing JayJay's compatibility overlay;
2. Tree-sitter's `DEP_TREE_SITTER_LANGUAGE_WASM_HEADERS` directory;
3. LLVM's compiler built-in headers.

The overlay files are based on Tree-sitter's shim rather than an Apple or Linux SDK:

| Header | Purpose |
| --- | --- |
| `assert.h` | Makes `__assert_fail` translation-unit-local to avoid duplicate linker symbols and supplies `static_assert`. |
| `ctype.h` | Extends Tree-sitter's `isprint` implementation with character functions used by scanners. |
| `string.h` | Extends Tree-sitter's declarations with string functions used by the selected grammars. |
| `wchar.h` | Supplies the missing `wchar_t` definition. |
| `wctype.h` | Extends Tree-sitter's wide-character helpers with functions used by scanners. |

Remove this overlay once the published Tree-sitter sysroot compiles and links JayJay's full grammar set without it.

## Host setup

The requirement is determined by the WASM target, not the build host:

| Build | Overlay required |
| --- | --- |
| Native macOS | No |
| Native Linux | No |
| Browser WASM built on macOS | Yes |
| Browser WASM built on Linux | Yes |

On macOS, install Homebrew LLVM because Apple Clang does not include the WebAssembly backend:

```bash
brew install llvm
```

On Linux, use a Clang/LLVM installation that includes the WebAssembly backend and `wasm-ld`. `scripts/llvm-clang` falls back to `clang` from `PATH` and verifies the backend. Override its selection when necessary:

```bash
clang --print-targets | grep wasm
JAYJAY_WASM_CLANG=/path/to/clang just test-wasm
```

## Verification

Install the Rust target, then run the linked build rather than relying on `cargo check`:

```bash
rustup target add wasm32-unknown-unknown
just test-wasm
```

The resulting debug module is `target/wasm32-unknown-unknown/debug/jayjay_uniffi.wasm`.

## Why not WASI SDK?

WASI SDK provides a complete C sysroot for WASI targets and is appropriate for standalone Tree-sitter grammar modules. JayJay instead links the grammar C code into its browser-oriented Rust `wasm32-unknown-unknown` module. Moving to `wasm32-wasip1` would add a WASI runtime contract and is therefore a target/runtime change, not a replacement compiler setup for the current build.
