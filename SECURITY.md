# Security Policy

## Supported Versions

Security fixes ship in the latest release on [GitHub Releases](https://github.com/hewigovens/jayjay/releases). Older builds and pre-releases are not patched; update to the current version first.

## Reporting a Vulnerability

Please do not open a public issue for security problems.

Report privately through [GitHub private vulnerability reporting](https://github.com/hewigovens/jayjay/security/advisories/new). Include the JayJay version, macOS or OS version, steps to reproduce, and what the issue lets an attacker do.

You will get an acknowledgement, and a fix or a decision, through the advisory thread. Please give us a reasonable window to release a fix before disclosing publicly.

## Scope

In scope: the JayJay app (SwiftUI and GPUI shells), the `jayjay` CLI, the Rust crates in this repository, and the release pipeline (build, notarization, appcast).

JayJay runs `jj`, `git`, `gh`, `glab`, and configured AI provider CLIs as subprocesses and reads repository contents, so reports where a repository, remote, or diff can run commands, escape the intended paths, or leak credentials are especially welcome.

Out of scope: vulnerabilities in `jj`, `git`, `gh`, `glab`, or other third-party tools themselves (report those upstream), and issues that require an already-compromised local account.
