# Review web test host

A minimal page that mounts [`@jayjay/review-web`](../../packages/review-web/README.md) with its local WASM backend so Playwright can exercise the real WASM, worker, and IndexedDB storage in Chromium and WebKit. It opens a five-file fixture change (`src/fixture.ts`) until a test imports another one. It is not a product surface; Control Room is the shipping consumer.

## Run

Prerequisites: the repository Rust toolchain, Node.js 22.12+ (or 20.19+), npm, Python 3, and LLVM Clang with a wasm32 target (`brew install llvm` on macOS).

```sh
rustup target add wasm32-unknown-unknown
rustup component add llvm-tools
cargo install --locked --version 0.2.129 wasm-bindgen-cli --root target/web-tools
npm ci
cd shell/web
npm run build                           # WASM packs, typecheck, production bundle
npx playwright install chromium webkit  # once
npm run test:browser
```

`npm run dev` serves the host with hot reload; Rust edits need `npm run wasm` again, and `npm run build:ui` reuses generated WASM after TypeScript or CSS changes. `WASM_BINDGEN=/path/to/wasm-bindgen` overrides the local tool; its version must match Cargo.lock.
