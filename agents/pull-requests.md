# Pull Request Workflow

Load this file before creating, updating, or landing pull requests. Follow the submission requirements in [CONTRIBUTING.md](../CONTRIBUTING.md), and load [Version Control Guide](version-control.md) before changing descriptions, history, or bookmarks.

JayJay publishes pull requests to GitHub from jj bookmarks. Keep each pull request focused on one logical change. Feature PRs leave the user guide, Help Book, website, and [shell-parity](shell-parity.md) matrix to the [release](release.md) docs pass. Explicit documentation requests follow the exception in [AGENTS.md](../AGENTS.md#user-facing-docs).

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

Identify the existing change to publish, its workspace, and the intended PR base. Work in that workspace and preserve the change as the publication target. For new implementation work, follow the [Feature Loop](../AGENTS.md#feature-loop) workspace recipe and fetch policy.

Apply the [commit and publication gates](../AGENTS.md#ready-to-commit-or-publish), including the two cleanup rounds. Select tests using [Testing](testing.md#running-tests); documentation-only changes use the documentation checks in the same policy. CI runs `swiftlint lint --strict`, so every SwiftLint warning that `just lint` prints fails the Lint Swift job; clear warnings, not just errors.

When publication is authorized under [Task Authority](../AGENTS.md#task-authority), describe the verified change, set a topic bookmark, and push it. These examples assume the intended change is `@`:

```bash
jj describe -m "summary" -m "body"
jj bookmark set <topic> -r @
jj git push --bookmark <topic>
```

Open the bookmark context menu in JayJay and choose **Pull Request on GitHub** or **Pull Request on Cursor**. For GitHub, `gh pr create --draft --base main --head <topic>` is also supported. For Cursor Origin, JayJay runs `origin pr create` when no PR exists for that bookmark. GitHub-mirrored Origin remotes cannot host Origin PRs; JayJay reports that error instead of opening the codebase page.

## Update after review

Locate the reviewed change in its workspace and apply authorized fixes there. Inspect adjacent cases for the same defect, but keep fixes within the requested scope; report unrelated findings separately. Follow [Scope And Convergence](code-review.md#scope-and-convergence), including re-checking the full patch for regressions. Use `jj diff --from <last pushed commit> --to @` to focus review on what changed since the previous push; identify the last pushed version by its immutable commit ID.

Rerun the affected checks after edits. When publication is authorized, apply the publication gates above and push the same bookmark with `jj git push --bookmark <topic>`. The bookmark follows the rewritten change. Fetch only when the task requests latest origin or a push reports that the remote bookmark moved; reconcile that movement before pushing again.

Hosted review-thread resolution also requires authorization under [Task Authority](../AGENTS.md#task-authority). When authorized, resolve each thread only after verifying its fix on the current pushed head; never blanket-resolve unresolved threads.

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
