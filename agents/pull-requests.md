# Pull Request Workflow

Load this file before creating, updating, landing, or documenting pull request workflows. Follow the submission requirements in [CONTRIBUTING.md](../CONTRIBUTING.md), and load [Version Control Guide](version-control.md) before changing descriptions, history, or bookmarks.

JayJay publishes pull requests to GitHub and Codeberg from jj bookmarks. Keep each pull request focused on one logical change.

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

Use `master@origin` or `trunk@origin` when that is the repository's trunk bookmark.

Before publishing, inspect the change, format it, and run the relevant tests and lint:

```bash
jj st
jj diff
jj fix
just test
just test-app  # SwiftUI changes
just test-ui   # user-visible SwiftUI workflows
just test-gpui # GPUI changes
just lint
```

Describe the change, set a topic bookmark, and push it:

```bash
jj describe -m "summary" -m "body"
jj bookmark set <topic> -r @
jj git push --bookmark <topic>
```

Open the bookmark context menu in JayJay and choose **Pull Request on GitHub**, **Pull Request on Codeberg**, or **Pull Request on Cursor**. For GitHub, `gh pr create --draft --base main --head <topic>` is also supported. For Cursor Origin, JayJay runs `origin pr create` when no PR exists for that bookmark. GitHub-mirrored Origin remotes cannot host Origin PRs; JayJay reports that error instead of opening the codebase page.

## Update after review

Fetch, edit the same change, apply the feedback, rerun the relevant checks, and push the same bookmark:

```bash
jj git fetch
jj edit <topic>

# edit, inspect, describe, format, and test

jj git push --bookmark <topic>
```

The bookmark follows the rewritten change. If the push reports that the remote bookmark moved, fetch and reconcile before pushing again.

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
