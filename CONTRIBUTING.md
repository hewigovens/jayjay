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
just build      # Build the macOS app
just run        # Build and launch the macOS app
just test       # Run Rust tests
just test-app   # Run Swift tests
just test-ui    # Run SwiftUI UI tests
just test-gpui  # Run GPUI component tests
just lint       # Run Clippy and SwiftLint
just format     # Run rustfmt and SwiftFormat
```

Run `just list` for the full command list.

## Testing

New features need focused unit coverage and UI flow coverage when behavior reaches the UI. Bug fixes should include the regression test that would have caught them. See the [testing guide](agents/testing.md) for test placement, fixtures, and UI-test conventions.

## Pull requests

Before publishing, run `jj fix`, the tests relevant to your change, and `just lint`. Write the change description as a concise summary, a blank line, and a body explaining what changed and why.

Publish changes by pushing a jj bookmark. See the [pull request workflow](agents/pull-requests.md) for creating, updating, stacking, and landing GitHub pull requests.

Pull requests for new UI features must include screenshots or a demo video so reviewers can evaluate the user-visible behavior.

## Documentation

When a feature lands:

- Update [README.md](README.md) when it changes what users can do.
- Update [Roadmap.md](Roadmap.md) when it changes planned or shipped status.
- Update this guide when it changes the contributor workflow.
