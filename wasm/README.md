# WASM build support

JayJay compiles its Rust core and Tree-sitter grammars into one `wasm32-unknown-unknown` module. The C header overlay in this directory is needed only for that cross-compilation; native macOS and Linux builds do not use it.

## Why the overlay exists

LLVM provides the WebAssembly code generator and compiler built-in headers, but `wasm32-unknown-unknown` has no corresponding C libc or platform sysroot. Host headers cannot be reused: macOS headers describe Darwin, and Linux headers describe the host's libc and ABI.

Tree-sitter Language 0.1.8 supplies a minimal target-specific libc shim through `DEP_TREE_SITTER_LANGUAGE_WASM_HEADERS`. JayJay's complete grammar set uses APIs missing from that published shim, so `scripts/llvm-clang` searches headers in this order:

1. `wasm/sysroot/include`, containing JayJay's compatibility overlay;
2. Tree-sitter's `DEP_TREE_SITTER_LANGUAGE_WASM_HEADERS` directory;
3. LLVM's compiler built-in headers.

`just test-wasm` resolves the shim path through Cargo metadata and supplies it as `JAYJAY_WASM_HEADERS` for older grammars such as Fish that do not depend directly on `tree-sitter-language` and therefore do not receive its header metadata.

The overlay files are based on Tree-sitter's shim rather than an Apple or Linux SDK:

| Header | Purpose |
| --- | --- |
| `assert.h` | Makes `__assert_fail` translation-unit-local to avoid duplicate linker symbols and supplies `static_assert`. |
| `ctype.h` | Extends Tree-sitter's `isprint` implementation with character functions used by scanners. |
| `stdlib.h` | Maps the Swift scanner's allocation-failure `exit` call to `abort` in the browser module. |
| `wctype.h` | Supplies `wchar_t` and `bool` for scanners that include only `wctype.h`. |

Tree-sitter Language 0.1.8 supplies `wchar_t` and declares the string and wide-character functions itself. Do not shadow its `wchar.h` or redeclare its functions as static inline helpers in the overlay; that hides required declarations or conflicts with them.

The C, C#, CSS, and Markdown grammar releases still compile the old per-grammar libc sources from `DEP_TREE_SITTER_LANGUAGE_WASM_SRC`. In 0.1.8 those paths contain deliberate errors for the obsolete build contract. For these grammars only, `scripts/llvm-clang` substitutes `wasm/grammar-stdlib.c`, an empty translation unit, because Tree-sitter 0.27 links the libc implementations once in its runtime. Remove this compatibility step when those grammar build scripts stop compiling the old sources.

Remove this overlay once the published Tree-sitter sysroot compiles and links JayJay's full grammar set without it.

## Host setup

The requirement is determined by the WASM target, not the build host:

| Build | Overlay required |
| --- | --- |
| Native macOS | No |
| Native Linux | No |
| Browser WASM built on macOS | Yes |
| Browser WASM built on Linux | Yes |

On macOS, check `clang --print-targets` for `wasm32`. If the installed Apple Clang lacks that backend, install Homebrew LLVM:

```bash
brew install llvm
```

On Linux, use a Clang/LLVM installation that includes the WebAssembly backend and `wasm-ld`. `scripts/llvm-clang` falls back to `clang` from `PATH` and verifies the backend. Override its selection when necessary:

```bash
clang --print-targets | grep wasm
JAYJAY_WASM_CLANG=/path/to/clang just test-wasm
```

## Verification

Install the Rust target and LLVM tools, then run the linked build rather than relying on `cargo check`. The recipe uses the toolchain's `llvm-ar`; Apple's `ar` can silently discard WASM objects:

```bash
rustup target add wasm32-unknown-unknown
rustup component add llvm-tools
just test-wasm
```

The resulting debug module is `target/wasm32-unknown-unknown/debug/jayjay_uniffi.wasm`.

Run `bash scripts/test-llvm-clang.sh` to check that legacy libc substitution is limited to the affected grammars and still propagates runtime and grammar compilation errors.

## Why not WASI SDK?

WASI SDK provides a complete C sysroot for WASI targets and is appropriate for standalone Tree-sitter grammar modules. JayJay instead links the grammar C code into its browser-oriented Rust `wasm32-unknown-unknown` module. Moving to `wasm32-wasip1` would add a WASI runtime contract and is therefore a target/runtime change, not a replacement compiler setup for the current build.
