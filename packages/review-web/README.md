# @jayjay/review-web

Reusable review UI and an optional local WASM backend for React 19 + Vite. This is a source package: the consuming app compiles its TypeScript, worker, and generated WASM assets. It is private and has not been published to a registry.

The Rust adapter in `wasm/` calls `jj-diff`, `jayjay-review`, and the shared file-tree builder. It owns diffing, fingerprints, marks, note anchors, and reconciliation. The React entry point owns navigation and presentation. It has no dependency on the local backend, IndexedDB, source imports, routing, or a service worker.

## Server backend

```tsx
import { ReviewWorkspace, useReview } from "@jayjay/review-web/react";
import type { Labels, ReviewBackend } from "@jayjay/review-web/types";
import "@jayjay/review-web/styles.css";

export function Review({ backend, labels, savedLabel }: { backend: ReviewBackend; labels: Labels; savedLabel: string }) {
  const review = useReview(backend);
  return <ReviewWorkspace review={review} labels={labels} savedLabel={savedLabel} />;
}
```

Keep the backend object stable (a module instance or `useMemo`). It implements three async methods, and a fourth optional one:

- `open(signal?)`: load the initial change and selected file, returning a `ReviewResult`.
- `render(selection, signal?)`: load a file with the requested options. Content can be fetched lazily here.
- `mutate(selection, action, signal?)`: apply a file/group mark or note action and return refreshed state after the durable write succeeds. Failures reject the promise; the viewer retains its previous result and offers Retry.
- `image(selection, side, signal?)`, optional: one side (`"old"` or `"new"`) of an image file as a `Blob`, or `null` when that side has none. With it, files whose extension `isImagePath` accepts (PNG, JPEG, GIF, WebP, BMP, ICO, SVG) show their before and after side by side, above their text diff. Give the blob an image type, since a blob opened as a page runs as the host's origin.

`ReviewSelection` contains an opaque numeric session handle, file index, and diff options. The adapter binds that handle to its immutable source revision. Control Room must retain and send the native expected commit when writing a Working-scope person mark. Reject stale sessions; a file or display-group index belongs to its pinned revision and whitespace mode. Branch comparisons remain outside the shared Working review contract.

Requests receive an abort signal and stale results never replace newer presentation. Check cancellation before beginning a write; aborting a request does not undo a write already accepted by the server. File/group actions set a desired state, so retries are idempotent. Notes need the backend's own retry semantics if a response can be lost after saving.

`ReviewWorkspace` accepts `headerActions` and `footer` slots. `useReview` exposes `run(signal => Promise<ReviewResult>)` for imports or refresh, plus selection, options, pending/error state, and Retry. Host copy goes through `savedLabel`; local-storage claims are never assumed. Labels come from JayJay's catalog or the host's localization adapter.

Styles are scoped to `.jayjay-review`, including form defaults. The optional `.jayjay-theme` class applies the palette and typography to a host root. A standalone `DiffPane` needs a `.jayjay-review` ancestor. The package does not register a service worker or style controls outside its root. [Class and variable names](#class-and-variable-names) lists what a host may theme or override.

## Class and variable names

Classes are `jj-` plus a role (`jj-file-row`, `jj-diff-cell`, `jj-token-Keyword`) and custom properties are `--jj-` plus a role (`--jj-accent`, `--jj-ink`), avoiding accidental collisions with a host's class and custom-property names. Host tag selectors and resets still apply. `.jayjay-review` and `.jayjay-theme` are the only unprefixed classes.

A host may rely on:

- `.jayjay-review` on an ancestor: every other rule needs it.
- `.jayjay-theme` on a host root: the palette and typography.
- `--jj-sticky-top`: the height of the host's own pinned bar, below which the compact list's path rows stick.
- The theme variables below, set on `.jayjay-review` or `.jayjay-theme`.

| Purpose | Variables |
| --- | --- |
| Surfaces | `--jj-page`, `--jj-surface`, `--jj-border` |
| Text | `--jj-ink`, `--jj-muted`, `--jj-mono` |
| Accent | `--jj-accent` |
| Added and removed lines | `--jj-added-ink`, `--jj-added-bg`, `--jj-removed-ink`, `--jj-removed-bg`, `--jj-word-added`, `--jj-word-removed` |
| File kinds | `--jj-kind-added`, `--jj-kind-removed`, `--jj-kind-modified` |
| Group stripes | `--jj-stripe-unreviewed`, `--jj-stripe-reviewed`, `--jj-stripe-changed`, `--jj-reviewed-ink` |
| Notes | `--jj-note-ink`, `--jj-note-bg`, `--jj-note-border`, `--jj-note-pill` |

Everything else is internal: it names structure the package owns, and may change in a release. Theme through the variables rather than styling a `jj-` class by name.

## Local WASM backend

```ts
import { LocalReviewClient } from "@jayjay/review-web/local";
import type { ReviewBackend } from "@jayjay/review-web/types";

const client = new LocalReviewClient({ databaseName: "my-review-app" });
const labels = await client.labels();
const backend: ReviewBackend = {
  open: async () => {
    const result = await client.open();
    if (!result) throw new Error("Import a review bundle first");
    return result;
  },
  render: selection => client.render(selection),
  mutate: (selection, action) => client.mutate(selection, action),
};
// client.open(JSON.stringify(bundle)) imports a validated full-text bundle.
// client.dispose() terminates its worker and rejects outstanding requests.
```

This entry point creates a dedicated module worker, loads the grammar-free base and requested syntax packs, and stores canonical review JSON in the named IndexedDB database. It also supplies `exportState`, `importState`, and guarded `restoreState`. Share a database name only when sharing source and review state on the same origin is intended. Syntax caches are disposable; source/state remain in IndexedDB.

Vite needs `worker: { format: "es" }`; TypeScript needs `jsx: "react-jsx"` and bundler module resolution. WASM packs get hashed URLs and load on demand. The host owns offline caching and installation. A server-only consumer importing `/react` and `/types` needs no WASM assets at runtime.

## Source layout

- `react.ts`, `local.ts`, `types.ts`: the package entry points.
- `model/`: the backend contract and the review and diff shapes it exchanges.
- `hooks/`: `useReview` (the review state machine) and `useCompact` (the narrow-screen layout switch).
- `components/`: `ReviewWorkspace` and its parts, grouped as `header/`, `files/`, `diff/`, and `notes/`.
- `local/`: the optional worker backend (`LocalReviewClient`, its worker protocol, WASM pack loading, and IndexedDB storage).
- `wasm/`: generated wasm-bindgen packs (ignored by version control).

## Build and consume

From the repository root:

```sh
npm ci
bash scripts/build-diff-web.sh
npm run check --workspace @jayjay/review-web
npm pack --workspace @jayjay/review-web --pack-destination /tmp
```

Install the tarball in another Vite app with `npm install /tmp/jayjay-review-web-0.0.0.tgz`. Build WASM before packing: generated `src/wasm/` assets are ignored by version control but included in the package. A source checkout dependency also needs that build. The Rust crate stays in the repository Cargo workspace.

`shell/web` is the browser test host for this package; see [its README](../../shell/web/README.md) to build and run the Playwright checks. Control Room owns its connected API adapter in its own repository.

## Review bundles

Import a UTF-8 JSON bundle (up to 32 MiB and 2,000 text files):

```json
{
  "schemaVersion": 1,
  "repositoryId": "owner/repository",
  "changeId": "stable-jj-change-id",
  "title": "Improve input handling",
  "description": "Optional change description",
  "author": "Optional author",
  "authorEmail": "Optional author email, for the avatar",
  "files": [
    {
      "path": "src/main.rs",
      "reviewIdentity": "opaque-native-review-identity",
      "before": "fn main() {}\n",
      "after": "fn main() { println!(\"hello\"); }\n"
    }
  ]
}
```

`oldPath` is optional for a rename. With `authorEmail` and **Load author avatars** turned on under Local review data, the header shows the author’s GitHub, GitLab or Gravatar avatar using the desktop app’s rules (`jayjay_primitives::avatar`); only then does the browser contact those services with the email’s GitHub id, GitLab username or SHA-256 hash. Paths are unique, repository-relative and `/`-separated. The repository ID scopes browser storage. Keep the actual jj change ID stable across rewrites and supply the current native `reviewIdentity` for each file. Do not substitute a commit ID or invent a different identity when sharing desktop review state. Reimporting a changed version uses canonical fingerprints to retain matching marks and invalidate changed groups; notes reconcile using their recorded identity and whitespace mode.

The integration boundary is the package’s typed `ReviewBackend` (`open`, `render`, `mutate`). A host can implement it with its server API and lazy per-file fetches without loading WASM or creating a local review store. The local backend, whose `open(JSON.stringify(bundle))` imports validated text bundles. No particular remote API, forge, authentication system, or repository filesystem bridge is assumed. The current loader imports complete text bundles; it computes and highlights only the selected file’s displayed diff. Binary/image review and server-side file-content streaming are not implemented.

## Storage and desktop compatibility

The local backend's IndexedDB database stores the last active bundle and one canonical `ReviewStore::snapshot_json()` document per repository. A mutation reads the latest document, calls Rust synchronously inside the IndexedDB write transaction, and acknowledges success only after commit. Independent tabs serialize their writes. File and group commands specify their desired state, so retrying a saved command is safe. The worker rejects requests for a superseded review session.

Local review data → Export review state produces the same JSON schema as desktop `review_store.json`. Import review state validates that format through Rust and replaces the current repository’s local state. A transaction keeps an undo snapshot; Undo import works until another edit changes that state. Unknown fields and entries survive later Rust mutations. Sharing state requires matching change IDs, paths, and native review identities; import/export does not automatically synchronize a desktop app or other devices.

Review bundles contain source code; review state contains paths, excerpts, and note bodies. Both remain on this device. Browser storage can be cleared or evicted, so exported review state is the portable backup.

## Native bridge

Control Room’s connected mode can link `jayjay-core` and `jayjay-review` directly. Shared marks cover **Working** scope (`@`); combined Branch comparisons stay read-only.

```rust
let review = jayjay_core::WorkingCopyReview::open(workspace)?;
let mut store = jayjay_review::ReviewStore::try_load()?;
let status = review.status(&store, false)?;
review.set_reviewed(&mut store, &jayjay_core::WorkingCopyReviewMark {
    expected_commit: status.change.commit_id.id.clone(),
    path: "src/main.rs".into(),
    display_group: None,
    ignore_whitespace: false,
    reviewed: true,
})?;
```

Run these synchronous native calls on a blocking worker in an async server. `files()` and `load_file(path)` support lazy file-content loading. The status result includes native file identities, canonical snapshots, display-group mappings, states, rollups, and ownership. A display-group index is valid only with its exact commit, path, and whitespace mode; pass `Some(index)` for a group action. Commands set a desired state and record a person’s ownership. Open a fresh session and checked store for each refresh; a retained session also rechecks the workspace commit before saving a mark. The bridge never snapshots or reconciles concurrent jj operations; multiple operation heads produce a retryable error. Agents’ unsnapshotted disk edits remain outside this immutable view.

All native review writers lock the same sidecar across reload and atomic save. Update the desktop app to a build using this locking implementation before running it alongside another writer. The Control Room server/API and connected PWA wiring live in the separate Control Room repository; this checkout supplies the library boundary, not that integration.
