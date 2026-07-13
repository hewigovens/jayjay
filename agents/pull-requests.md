# Pull Request Workflow

Load this file before creating, updating, landing, or documenting pull request workflows.

JayJay uses jj bookmarks for pull requests on both GitHub and Codeberg. This keeps the pushed commit identical to the signed jj change, avoids generated PR commits, and matches jj's normal edit-and-rewrite model.

GitHub PR status and checks use `gh`. Public Codeberg PR status and commit statuses use the unauthenticated Forgejo API; private Codeberg repositories are not integrated yet.

## Change Descriptions

Write every change description as a **summary line, a blank line, then a body** — the same split as GitHub Desktop's commit box and JayJay's own two-field commit box:

- **First line** — a concise, imperative summary (~50–72 chars). This becomes the PR/MR **title**, and for stacked PRs each layer's title.
- **Blank line**, then the **body** — what changed and why. This becomes the PR/MR **body**.

The stacked-PR engine derives the title from the first line and the body from the rest (`jayjay_core::commit_message::{summary, body}`), so a one-line description ships a one-line PR title with an empty body. Write the body even for small changes.

Pass the two parts as separate `-m` flags (jj joins them with a blank line):

```bash
jj describe -m "feat(diff): wrap long lines in the unified view" \
  -m "Soft-wrap at the column width instead of truncating so reviews of generated files stay readable. Adds a per-pane toggle persisted in settings."
```

## Default Flow

Start from current trunk:

```bash
jj git fetch
jj new main@origin
```

Keep edits in the working-copy change `@` until they are ready to publish:

```bash
jj st
jj diff
jj describe -m "concise summary line" -m "body: what changed and why"
```

Split by responsibility when the working copy contains more than one logical change:

```bash
jj split <fileset-for-one-change> -m "one logical change"
```

Repeat `jj split` until each PR-sized change has one clear purpose. Do not split just to mirror file boundaries; split by behavior, bug fix, or user-visible feature.

Before publishing, format and run the checks that match the change:

```bash
jj fix      # run configured formatters (rustfmt, SwiftFormat) over the change
just test
just test-app
just lint
```

Publish the selected change by moving a bookmark to it and pushing that bookmark:

```bash
jj bookmark set <topic> -r @
jj git push --bookmark <topic>
```

Then open the bookmark context menu in JayJay and choose **Pull Request on GitHub** or **Pull Request on Codeberg**. For GitHub, `gh pr create --draft --base main --head <topic>` is also fine when the browser flow is inconvenient.

Use `master@origin` or `trunk@origin` instead of `main@origin` when that is the repository's trunk bookmark.

## Review Updates

Handle review feedback by editing the same change and pushing the same bookmark again:

```bash
jj git fetch
jj edit <topic>

# edit files
jj st
jj diff
jj describe -m "updated summary line" -m "updated body"
jj fix
just test
just lint

jj bookmark set <topic> -r @
jj git push --bookmark <topic>
```

The remote branch moves as part of normal jj history editing. `jj git push` applies jj's bookmark safety checks, so fetch first if the push reports that the remote bookmark changed.

## Multiple Changes

For independent PRs, use one bookmark per ready change:

```bash
jj bookmark set <topic-a> -r <rev-a>
jj bookmark set <topic-b> -r <rev-b>
jj git push --bookmark <topic-a>
jj git push --bookmark <topic-b>
```

For stacked work, prefer separate bookmarks only when the stack is actually useful for review. Push the base change first, then the dependent change, and set the dependent PR's base branch to the base bookmark in the hosting UI.

## Landing Cleanup

After the PR lands:

```bash
jj git fetch
jj new main@origin
```

If the hosting service deleted the remote branch, forget the local bookmark and its stale remote tracking state:

```bash
jj bookmark forget <topic> --include-remotes
```

If the hosting service did not delete the remote branch, delete it before forgetting it:

```bash
jj bookmark delete <topic>
jj git push --bookmark <topic>
```

Keep the local reviewed change only when it is still useful for follow-up work; otherwise start new work from `main@origin`.
