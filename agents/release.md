# Release Workflow

Load this file before version bumps, packaging, appcast changes, GitHub releases, or Homebrew tap updates.

Releases are not complete after `just release`. The full release flow is:

1. Bump version and build number in all four sources: `shell/mac/project.yml`, `shell/mac/JayJay.xcodeproj/project.pbxproj`, `crates/jayjay-cli/Cargo.toml`, and `shell/justfile`.
2. Write release notes to `releases/<version>.html` as an HTML body without wrapper tags.
3. Run `just build` to verify the release version still builds.
4. Run `just release` to build, sign, notarize, zip, verify the extracted archive with `codesign`, `stapler validate`, and `spctl -av`, produce the SHA-256, and prepend the entry to `docs/appcast.xml`.
5. Commit the version bumps, `releases/<version>.html`, and `docs/appcast.xml` as `release: <version> (build N)`.
6. Create and push the `v<version>` tag from the release commit.
7. Run `just shell::publish` to create the public GitHub release, upload the zip, verify the Sparkle asset URL is public, and rewrite `../tap/Casks/jayjay.rb`.
8. Push `main` only after `just shell::publish` succeeds, so `docs/appcast.xml` never points at a missing or draft-only asset.
9. Commit and push the Homebrew tap change in `../tap`.

## Required Outputs

- `just release` produces the notarized zip in `build/release/`.
- The GitHub release must include the zip asset and its SHA-256.
- `docs/appcast.xml` must match the uploaded release asset and include a `<description>` block sourced from `releases/<version>.html`.
- `../tap/Casks/jayjay.rb` must match the uploaded release asset and SHA-256.

## Release Notes

Release notes are mandatory. `update-appcast.py` reads `releases/<version>.html` and embeds it in the appcast description. A release without notes ships an empty Sparkle update prompt, which is not acceptable.
