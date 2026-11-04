# Pull request review

Renders any public GitHub pull request with [`@jayjay/review-web`](../../web/review/README.md): open `/<owner>/<repo>/pull/<number>`, or swap `github.com` for this site's host in a pull request URL. Review marks and notes stay in the browser's IndexedDB, keyed by `github.com/<owner>/<repo>` and `pull/<number>`, so they carry across pushes the same way desktop marks carry across rewrites.

A Cloudflare Worker (`worker/`) answers `/api/<owner>/<repo>/pull/<number>` with the pull request's title, author, merge base, head, and file list with line counts and blob IDs, cached at the edge for 60 seconds. The file list comes from the pull request's `.diff` on github.com, which is outside the API rate limit, so a pull request takes the same few REST calls however many files it changes. The browser opens the review as soon as the first file's two sides arrive from raw.githubusercontent.com, which is outside the API rate limit; it fetches a file on demand when it is selected and every other file in the background. A file's review identity is its two blob IDs, so marks apply before its content loads. Everything else is static assets. A service worker (`sw.js`, versioned at build time) caches the page and its scripts, and the diff engine keeps its WASM packs in Cache Storage, so repeat visits load nothing but the pull request until the next deploy. `src/styles/page.css` sets the viewer's palette through the package's `--jj-` theme variables.

## Run

Build the WASM packs once as described in [web/review-host](../web/README.md#run), then from the repository root:

```sh
npm ci
cd web/diff
npm run dev        # Vite with the Worker in workerd; no service worker
npx vite build && npx vite preview   # the production build, with the service worker
npm test           # Vitest: URL parsing, GitHub mapping, bundle building
npm run check      # typecheck app and Worker, oxlint
```

## Deploy

When the site's limit runs out, the page offers **Sign in with GitHub**: a device-code sign-in through the JayJay OAuth app (`GITHUB_CLIENT_ID` in `wrangler.jsonc`), relayed by `/api/login/*` since GitHub's endpoints have no CORS. The token asks for no scope, stays in the reader's browser, and goes with each `/api` request so the reader spends their own 5,000-an-hour limit; the Worker refuses private repositories, so nothing private reaches the shared edge cache.

The sync icon turns on **Viewed sync**: it signs in again asking for `public_repo`, which GitHub's `markFileAsViewed` mutation needs, and the browser then talks to GitHub's GraphQL API directly with that token (`src/github/ViewedSync.ts`). On each open, files checked Viewed on GitHub are marked reviewed here; marking a whole file reviewed here, or a hunk that completes or reopens it, checks or clears its box on GitHub. Turning sync on also checks the files already reviewed here. Unchecking a box on GitHub doesn't clear a local mark, and hunk marks have no GitHub equivalent. Signing out stops syncing.

The commit icon lists a pull request's commits, newest first, when it has more than one. Picking one opens `/<owner>/<repo>/pull/<number>/commits/<sha>`, GitHub's own URL for it; the Worker answers that path under `/api` with the commit diffed against its first parent, and only for the pull request's own commits. A commit's marks and notes are its own, keyed by its SHA, and Viewed sync pauses there, since GitHub's boxes belong to the whole pull request.

Pull requests stacked through their base branches, each based on the head branch of the one below, as `gh stack` and jj bookmark stacks push them, get a stack icon listing them top first. The Worker follows the chain from the open pull request both ways with one `pulls?head=` or `pulls?base=` call a step, stops where a step finds none or several, and leaves `stack` out of the manifest when there's nothing to follow.

The landing page's sample pull requests (`src/github/samples.ts`) keep opening when the rate limit is spent: the Worker stores each one's manifest in the `STORE` KV namespace the first time it fetches it and serves that copy from then on. Pick closed or merged pull requests, which no longer change; delete a `sample:` key to refresh it.

**Share notes with an agent** posts the open notes and review marks to `/api/exports`, which keeps them in the `STORE` KV namespace for 7 days under a random 128-bit ID and copies the link. `GET /api/exports/<id>` serves Markdown for an agent and `/api/exports/<id>.json` the raw export. Anyone with the link can read it.

`GITHUB_TOKEN` is optional: without it the Worker gets GitHub's unauthenticated 60 requests an hour per egress IP. Use a fine-grained token with no repository permissions; it only raises the limit to 5,000 an hour.

```sh
npx wrangler secret put GITHUB_TOKEN
npm run deploy
```

`wrangler.jsonc` serves it at `diff.hewig.dev`, a custom domain on the `hewig.dev` Cloudflare zone; the first deploy creates the DNS record and certificate.

Limits: public repositories only; at most 2,000 files; binary files other than images review as empty text.
