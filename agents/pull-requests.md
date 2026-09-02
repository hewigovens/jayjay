# Pull Request Workflow

Load this file before creating, updating, or landing pull requests. Follow the submission requirements in [CONTRIBUTING.md](../CONTRIBUTING.md), and load [Version Control Guide](version-control.md) before changing descriptions, history, or bookmarks.

JayJay publishes pull requests to GitHub from jj bookmarks. Keep each pull request focused on one logical change. Do not update the user guide, Help Book, website, or [shell-parity](shell-parity.md) matrix here — that is the [release](release.md) docs pass.

## Describe the change

Write every change description as a concise, imperative summary, a blank line, and a body explaining what changed and why. JayJay uses the first line as the pull request title and the rest as its body.

Pass the parts as separate `-m` flags so jj inserts the blank line:

```bash
jj describe -m "feat(diff): wrap long lines" \
  -m "Soft-wrap long lines so generated files remain readable."
```

Do not leave the body empty, even for small changes. Split unrelated work by behavior or responsibility, not by file boundary:

```bash
jj split <fileset> -m "summary" -m "body"
```

## Publish

Start new work from the current trunk:

```bash
jj git fetch
jj new main@origin
```

Use `master@origin` or `trunk@origin` when that is the repository's trunk bookmark. Prefer a sibling workspace for the implementation itself; see [Version Control](version-control.md).

Before publishing, finish the two cleanup rounds from `AGENTS.md`, then inspect the change, format it, and run the tests that match what changed — not the whole matrix:

```bash
jj diff
jj fix
just test-rust <crate>          # Rust crate change
just test-app                   # SwiftUI app change
just test-ui JayJayUITests/…    # user-visible SwiftUI workflow
just test-gpui                  # GPUI-only change; skip if just test-rust already ran jayjay-gpui
just lint
```

`just test` (`cargo test --workspace`) is the full Rust gate when several crates moved. Do not also run `just test-gpui`. Do not run `just build` unless the change is the macOS app bundle or UniFFI packaging. CI runs `swiftlint lint --strict`, so every SwiftLint warning that `just lint` prints fails the Lint Swift job; clear warnings, not just errors.

Describe the change, set a topic bookmark, and push it:

```bash
jj describe -m "summary" -m "body"
jj bookmark set <topic> -r @
jj git push --bookmark <topic>
```

Open the bookmark context menu in JayJay and choose **Pull Request on GitHub** or **Pull Request on Cursor**. For GitHub, `gh pr create --draft --base main --head <topic>` is also supported. For Cursor Origin, JayJay runs `origin pr create` when no PR exists for that bookmark. GitHub-mirrored Origin remotes cannot host Origin PRs; JayJay reports that error instead of opening the codebase page.

## Update after review

Fetch, edit the same change, apply the feedback, rerun the relevant checks, and push the same bookmark:

```bash
jj git fetch
jj edit <topic>

# edit, inspect, describe, format, and test

jj git push --bookmark <topic>
```

The bookmark follows the rewritten change. If the push reports that the remote bookmark moved, fetch and reconcile before pushing again.

Fix the whole neighbourhood of a finding in one pass — the symmetric case, the other side of the comparison, the sibling code path, the async-init race — because the next round probes exactly there. Before pushing, review only the delta since the last push (`jj diff --from <last pushed commit> --to @`) as adversarially as the reviewer would. Resolve review threads one at a time, each after verifying that thread's fix is on the pushed head; never blanket-resolve everything unresolved.

## Multiple changes

Use one bookmark per independent pull request. For stacked work, create separate bookmarks only when the dependency is useful to reviewers. Push the base first, push the dependent change second, and set the dependent pull request's base branch to the base bookmark in the hosting UI.

## After landing

Fetch and start new work from the updated trunk:

```bash
jj git fetch
jj new main@origin
```

If the hosting service deleted the remote branch, forget the local and remote-tracking bookmark:

```bash
jj bookmark forget <topic> --include-remotes
```

Otherwise, delete the bookmark and push the deletion:

```bash
jj bookmark delete <topic>
jj git push --bookmark <topic>
```
