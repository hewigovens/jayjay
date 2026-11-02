# TypeScript Guide

Load this file before changing TypeScript, React, or CSS under `web/`. Crate boundaries live in [Architecture Guide](architecture.md); browser test placement in [Testing Guide](testing.md#browser-review-tests). Repository-wide rules in `AGENTS.md` (comments, tests, code organization) apply here too; this guide adds what is specific to the web code.

## File Layout

```text
web/
├── review/        @jayjay/review-web, the reusable review UI
│   ├── src/
│   │   ├── components/   ReviewWorkspace plus diff/, files/, header/, notes/ by area
│   │   ├── hooks/        React state: useReview, useCompact, workspace hooks
│   │   ├── model/        backend contract and data types, React-free
│   │   ├── local/        optional worker + IndexedDB backend
│   │   ├── styles/       one stylesheet per area, imported by styles.css
│   │   └── react.ts, local.ts, types.ts, styles.css   package entry points
│   └── wasm/      jj-diff-web crate: wasm-bindgen bindings, no business logic
├── review-host/   Playwright host for the package: src/ mounts it, tests/ one spec per workflow
└── diff/          JayJay Diff (diff.hewig.dev)
    ├── src/
    │   ├── components/   landing/, review/, sign-in/ by screen
    │   ├── hooks/        React state for sign-in and the open review
    │   ├── styles/       page, review, sign-in, landing stylesheets, imported by style.css
    │   └── github/       review references, manifests, loading, sign-in, Viewed sync, React-free
    ├── worker/    Cloudflare Worker: routes in index.ts, github/ API client and diff parsing
    └── tests/     Vitest, one file per module under test
```

## Tooling

- oxfmt formats `.ts`, `.tsx`, `.js`, and `.css` at 100 columns: `npm run format` at the root, `just format`, or `jj fix`. CI runs `npm run format:check`.
- Each workspace's `npm run check` runs `tsc` in strict mode (with `noUnusedLocals` and `noUnusedParameters`) and oxlint; `just lint` runs them all.
- Vitest covers React-free logic in `web/diff`; Playwright in `web/review-host` covers the package in Chromium and WebKit; `cargo test -p jj-diff-web` covers the WASM adapter.

## Files and Folders

- One primary export per file, named after it: `PascalCase.tsx` for a component, `PascalCase.ts` for a class or a type with its vocabulary, `useThing.ts` for a hook, `camelCase.ts` for a module of related functions, types, or constants (`model/review.ts`, `samples.ts`).
- Folders are lowercase and named for a responsibility (`components/files`, `hooks`, `github`). No folder for a single file, and no `index.ts` barrels except a package's declared entry points.
- Split when a file takes on a second job (a hook beside its component, a parser beside a client), not by line count.
- Keep domain logic React-free: parsing, GitHub mapping, and review state live in plain modules that Vitest can import; components render state and dispatch actions; hooks own effects and subscriptions.

## Code

- No `any`. Data crossing a boundary (fetch, `postMessage`, storage, JSON) arrives as `unknown` and is narrowed or validated before use.
- `import type` for type-only imports. Named exports only; a default export only where a tool requires one (Vite, Vitest, Playwright configs and the Worker entry).
- `interface` for object shapes, `type` for unions, aliases, and mapped types.
- Behavior that belongs to a type is a class or an object with methods (`PullLoader`, `GitHubSignIn`); pure transforms are functions.
- Cancellable work takes an `AbortSignal`. A promise is awaited, returned, or explicitly discarded with `void`.
- Errors carry context (`HTTP 404 for …`), and a caught error is either handled or rethrown, never dropped silently.
- User-facing text comes from the Fluent catalog through `labels`, never a string literal in a component. Tool output meant for agents (Markdown exports) stays English.

## React

- Function components only; a component's props interface sits beside it as `<Component>Props` when it is exported or long, inline otherwise.
- Keys are stable identities (paths, ids), never array indexes for reorderable lists.
- Preserve keyboard access and focus: every action has a button or shortcut, dialogs return focus, and lists keep the focused row when they re-render.

## CSS

- Plain CSS with custom properties. The package prefixes its classes with `jj-` and reads only its own `--jj-*` tokens, so a host's styles never leak in.
- Inline `style` only for computed values (widths, positions).

## Comments

`AGENTS.md` principle 4 applies unchanged: no comment by default, one line when the reason is not evident from the code. TSDoc (`/** */`) is reserved for a package's public API where the type alone leaves the contract unclear, such as units, what `null` means, or when a callback runs. Pragmas such as `/** @jsxRuntime automatic */` and `/// <reference lib="webworker" />` are directives, not comments.
