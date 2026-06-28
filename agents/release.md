# Release Workflow

Load this file before version bumps, packaging, appcast changes, GitHub releases, or Homebrew tap updates.

Releases are not complete after `just release`. The full release flow is:

1. Run `just set-version <version> <build>` to bump every source at once (`shell/justfile` version + build_number, `crates/jayjay-cli/Cargo.toml`, `shell/mac/project.yml`). `project.pbxproj` and `Cargo.lock` regenerate on build. Never hand-edit one source — the build number lives in two files and drift ships a broken update.
2. Write release notes to `releases/<version>.html` as an HTML body without wrapper tags.
3. Run `just build` to verify the release version still builds.
4. Run `just release` to build, sign, notarize, zip, verify the extracted archive with `codesign`, `stapler validate`, and `spctl -av`, produce the SHA-256, and prepend the entry to `docs/appcast.xml`. It first runs `just check-version`, aborting if any source disagrees. Keep the Mac unlocked: a locked screen locks the keychain, so notarization fails with `No Keychain password item found for profile: notarytool` even when the profile exists.
5. Commit the version bumps, `releases/<version>.html`, and `docs/appcast.xml` as `release: <version> (build N)`.
6. Create and push the `v<version>` tag from the release commit. Tag pushes run the AppImage workflow and retain the Linux alpha builds as CI artifacts.
7. Run `just shell::publish` to create the public GitHub release, upload the zip, verify the Sparkle asset URL is public, and rewrite `../tap/Casks/jayjay.rb`. The AppImage workflow runs when that release is published and attaches GPUI Linux alpha AppImages plus SHA-256 files asynchronously.
8. Push `main` only after `just shell::publish` succeeds, so `docs/appcast.xml` never points at a missing or draft-only asset.
9. Commit and push the Homebrew tap change in `../tap`.

The Cloudflare worker is transparent to appcast generation: `docs/appcast.xml` remains the source of truth, and the worker only proxies it for users who opt into anonymous stats. If a release changes `infra/worker`, the worker name, or any `workers.dev` endpoint, deploy it before shipping and verify both routes:

```bash
cd infra/worker
wrangler deploy
curl -fsS https://jayjay.hewigovens.workers.dev/appcast.xml >/dev/null
curl -fsS 'https://jayjay.hewigovens.workers.dev/ping?probe=1&platform=gpui&app=jayjay&version=test&os=darwin&arch=arm64'
```

## Required Outputs

- `just release` produces the notarized zip in `build/release/`.
- The GitHub release must include the zip asset and its SHA-256.
- For GPUI Linux alpha releases, the AppImage workflow attaches `jayjay-gpui-x86_64-linux.AppImage`, `jayjay-gpui-aarch64-linux.AppImage`, and matching `.sha256` files after the release is published.
- `docs/appcast.xml` must match the uploaded release asset and include a `<description>` block sourced from `releases/<version>.html`.
- `../tap/Casks/jayjay.rb` must match the uploaded release asset and SHA-256.

## Release Notes

Release notes are mandatory. `update-appcast.py` reads `releases/<version>.html` and embeds it in the appcast description. A release without notes ships an empty Sparkle update prompt, which is not acceptable.
