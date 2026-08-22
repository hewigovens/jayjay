# Release Workflow

Load this file before version bumps, packaging, appcast changes, GitHub releases, or Homebrew tap updates.

Releases are not complete after `just release`. The full release flow is:

1. Start a release change directly on published `main`, then run `just set-version <version> <build>` to bump every source at once (`shell/justfile` version + build_number, the root Cargo workspace package version inherited by the CLI and GPUI, and `shell/mac/project.yml`). `project.pbxproj` and `Cargo.lock` regenerate on build. Never hand-edit one source — version drift ships binaries and update metadata that disagree. Both release recipes refresh `origin` and abort unless the release change's parent, local `main`, and `main@origin` are the same commit; publication repeats the check against the release tag.
2. Diff the complete range from the previous release tag with `jj log -r 'v<previous>..@'` and `jj diff --from v<previous> --to @ --summary`. From that range, refresh **shipped user docs** (below) and write SwiftUI macOS notes as an HTML body without wrapper tags in `releases/<version>.html`. Cover user-visible SwiftUI changes from the whole range, not only the current local stack.
3. Run `just build` to verify the release version still builds.
4. Run `just release` to verify immutable worker migration checksums, build, sign, notarize, zip, verify the extracted archive with `codesign`, `stapler validate`, and `spctl -av`, produce the SHA-256, and prepend the entry to `docs/appcast.xml`. It also runs `just check-version`, aborting if any source disagrees. Keep the Mac unlocked: a locked screen locks the keychain, so notarization fails with `No Keychain password item found for profile: notarytool` even when the profile exists.
5. Commit the version bumps, shipped user docs, SwiftUI release notes, and `docs/appcast.xml` as `release: <version> (build N)`.
6. Create and push the `v<version>` tag from the release commit. Tag pushes run the AppImage workflow and retain the Linux alpha builds as CI artifacts, but the GPUI alpha artifacts are not a release gate.
7. Run `just shell::publish` to create the public GitHub release, upload the zip, verify the Sparkle asset URL is public, and rewrite `../tap/Casks/jayjay.rb`. The AppImage workflow runs when that release is published and attaches GPUI Linux alpha AppImages plus SHA-256 files asynchronously; do not wait for it during the macOS release unless you are specifically validating GPUI alpha artifacts.
8. Push `main` only after `just shell::publish` succeeds, so `docs/appcast.xml` never points at a missing or draft-only asset.
9. Commit and push the Homebrew tap change in `../tap`.

## Beta Releases

Beta versions use the `<X.Y.Z>-beta.<N>` scheme with a monotonic build number; the stable release that follows must use a higher build number (the appcast script enforces this). The suffix exists only in release artifacts — the tag, zip, notes file, appcast item title, and channel tag. The installed app carries the base `X.Y.Z` in `MARKETING_VERSION` and the Cargo version, so beta builds report plain versions to telemetry and need no worker or D1 changes; identify a beta cohort by its build number.

`just set-version 0.3.16-beta.1 50` writes the split automatically, `just release` tags the appcast item with `<sparkle:channel>beta</sparkle:channel>` (invisible to clients that have not selected the Beta update channel in Settings > About) and prepends a beta banner to the item's description because the update prompt shows only the base version, and `just shell::publish` marks the GitHub release as a pre-release and skips the Homebrew cask. Betas still require `releases/<version>.html` notes and the full notarization flow. Promote by cutting the plain stable version with the next build number; do not edit a published beta's appcast entry in place.

The Cloudflare worker is independent of Sparkle: `docs/appcast.xml` remains the direct update source, while enabled anonymous statistics use `/ping`. `/appcast.xml` remains only for compatibility with older releases, whose payloads the worker must continue to accept. If a release changes `infra/worker`, the worker name, the telemetry payload, or any `workers.dev` endpoint, apply pending D1 migrations before deploying the worker, then verify both routes:

```bash
just worker::deploy
curl -fsS https://jayjay.hewigovens.workers.dev/appcast.xml >/dev/null
curl -fsS 'https://jayjay.hewigovens.workers.dev/ping?probe=1&platform=gpui&app=jayjay&version=test&os=darwin&arch=arm64'
```

`just worker::deploy` verifies immutable migration checksums, applies pending
migrations, confirms the remote D1 ledger matches the local migration set, and
only then uploads worker code. Existing migration files and checksum lines are
immutable; add the next numbered migration and checksum for every schema change.

## Required Outputs

- `just release` produces the notarized zip in `build/release/`.
- `just release` also preserves `build/release/JayJay-<version>.dSYM.zip`; keep it for crash-log symbolication because release binaries are stripped before signing.
- The GitHub release must include the zip asset and its SHA-256.
- For GPUI Linux alpha releases, the AppImage workflow attaches `jayjay-gpui-x86_64-linux.AppImage`, `jayjay-gpui-aarch64-linux.AppImage`, and matching `.sha256` files after the release is published. This is asynchronous and non-blocking for the macOS release.
- `docs/appcast.xml` must match the uploaded release asset and include a `<description>` block sourced only from the SwiftUI notes in `releases/<version>.html`.
- The GitHub release body uses `releases/<version>.html`. GPUI builds live on the same tag and GitHub release as asynchronously attached artifacts, without a repository release-notes file.
- `../tap/Casks/jayjay.rb` must match the uploaded release asset and SHA-256.

## Shipped User Docs

Feature PRs do not update the user guide, Help Book, website, or parity matrix. Do that once per release from `v<previous>..@`. Skip a file whose topic did not ship in this range. Canonical sources, in order:

- `docs/guide.html` — public user guide (workflow source of truth)
- `docs/imgs/` — screenshots shared with the Help Book
- `shell/mac/Resources/JayJayHelpBook/` — topic HTML reused from the guide; rebuild with `just shell::help`
- `docs/llms.txt` — machine-readable feature summary, only if the feature list changed
- `docs/index.html` FAQ — only if the range answers a common question
- `agents/shell-parity.md` — matrix rows aligned to the guide, including closed gaps
- `README.md` — only install, positioning, or requirements changes; do not duplicate the guide
- `Roadmap.md` — shipped vs planned status
- `UserGuide.md` — stub pointing at the web guide; do not grow a second copy of the guide

Do not edit `docs/appcast.xml` in this pass; step 4 owns it. Load [Help Book Guide](help-book.md) before changing Help pages or `docs/guide.html`.

## Release Notes

`releases/<version>.html` is mandatory, covers only the SwiftUI macOS app, and is the source for both the Sparkle update prompt and GitHub release body. GPUI releases are represented by the shared tag and asynchronously attached GitHub release artifacts instead of a separate notes file. Missing or empty SwiftUI notes abort publication.
