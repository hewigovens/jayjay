# Review web test host

A minimal page that mounts [`@jayjay/review-web`](../review/README.md) with its local WASM backend so Playwright can exercise the real WASM, worker, and IndexedDB storage in Chromium and WebKit. It opens a five-file fixture change (`src/fixture.ts`) until a test imports another one. It is not a product surface; Control Room is the shipping consumer.

## Run

Prerequisites: the repository Rust toolchain, `cmake`, Node.js 22.12+ (or 20.19+), npm, and LLVM Clang with a wasm32 target (`brew install llvm` on macOS). The first build downloads the tree-sitter CLI and its WASI SDK to compile grammars.

```sh
rustup target add wasm32-unknown-unknown
rustup component add llvm-tools
cargo install --locked --version 0.2.129 wasm-bindgen-cli --root target/web-tools
npm ci
cd web/review-host
npm run build                           # engine, grammars, typecheck, production bundle
npx playwright install chromium webkit  # once
npm run test:browser
```

`npm run dev` serves the host with hot reload; Rust edits need `npm run wasm` again, and `npm run build:ui` reuses generated WASM after TypeScript or CSS changes. `WASM_BINDGEN=/path/to/wasm-bindgen` overrides the local tool; its version must match Cargo.lock. Grammars that jj-diff does not bundle are compiled once per source version into `target/web-grammars` (`JAYJAY_GRAMMAR_CACHE` overrides it). `node scripts/web-grammar-parity.mjs` (Node.js 22.18+, which runs TypeScript directly), run from the repository root after a build, checks that every language highlights the same on the web as natively.
