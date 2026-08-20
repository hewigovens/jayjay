# Contributing to JayJay

JayJay uses [Jujutsu](https://github.com/jj-vcs/jj) for version control, not git.

## Before you start

- **New features:** [Open an issue](https://github.com/hewigovens/jayjay/issues/new) before starting implementation so the proposal can be discussed and tracked.
- **Bug fixes:** You may submit a pull request directly. Link an existing issue when one is available and describe the bug and fix in the pull request.

## Setup

Building the macOS app requires macOS 26+, Rust 1.96+, and Xcode 16+. Bootstrap the remaining tools and configure `jj fix` with:

```bash
./scripts/setup.sh
```

The script installs Homebrew and Rust when needed, installs the tools in [Brewfile](Brewfile), and initializes a colocated jj workspace. Xcode must be installed separately.

## Development

Business logic belongs in the Rust core. The SwiftUI and GPUI shells render state and dispatch actions. Read [AGENTS.md](AGENTS.md) and its focused guides before making architectural or shell-specific changes.

Common commands:

```bash
just test-rust <crate>   # Package-scoped Rust tests (inner loop)
just test-ui <test-id>   # One XCUITest scene
just test                # All workspace Rust tests (publish)
just test-app            # Swift unit tests
just test-gpui           # GPUI component tests
just lint                # Clippy + SwiftLint (publish)
just format              # rustfmt + SwiftFormat (publish)
just build               # macOS app (not the inner loop)
just run                 # Build and launch
```

Run `just list` for the full command list. See [AGENTS.md](AGENTS.md) for the feature loop: sibling jj workspaces, delayed lint/format, and which docs wait for release.

## Testing

New features need focused unit coverage and UI flow coverage when behavior reaches the UI. Bug fixes should include the regression test that would have caught them. See the [testing guide](agents/testing.md) for test placement, fixtures, and UI-test conventions.

## Pull requests

Before publishing, do the two cleanup rounds from [AGENTS.md](AGENTS.md) (dedupe, simplify, delete what the change left dead), then run `jj fix`, the tests relevant to your change, and `just lint`. Write the change description as a concise summary, a blank line, and a body explaining what changed and why.

Publish changes by pushing a jj bookmark. See the [pull request workflow](agents/pull-requests.md) for creating, updating, stacking, and landing GitHub pull requests.

Pull requests for new UI features must include screenshots or a demo video so reviewers can evaluate the user-visible behavior.

## Documentation

User-facing docs (the [web guide](https://jayjay.hewig.dev/guide.html), Help Book, FAQ, `docs/llms.txt`, README feature lists, Roadmap, and the shell-parity matrix) update in the [release](agents/release.md) shipped-docs pass, not in feature PRs.

Update this contributing guide when the **contributor** workflow changes. Update `agents/*.md` in a feature change only when the agent/contributor contract actually changed.
