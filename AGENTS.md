# Agent Instructions for jayjay

## Version Control

This repo uses **Jujutsu (jj)**, not git. All version control operations should use `jj` commands.

### Key differences from git

- **No staging area**: jj auto-snapshots the working copy. Every change is always "committed".
- **Changes, not commits**: jj uses "changes" identified by change IDs (reverse hex), not commit hashes.
- **`@` is the working copy**: always refers to the current working copy change.
- **`@-` is the parent**: the previous change.
- **Mutable history**: all changes are rewritable, not just the latest.

### Common commands

```bash
# Status
jj st                          # Working copy status
jj log                         # DAG history
jj log --limit 10              # Recent changes
jj diff                        # Working copy diff
jj diff -r <rev>               # Diff for a specific change

# Making changes
jj describe -m "message"       # Set description for @
jj commit -m "message"         # Describe @ and start new empty change
jj new                         # Start new empty change on top of @

# History manipulation
jj squash                      # Squash @ into parent
jj squash -r <rev>             # Squash a specific change into its parent
jj abandon <rev>               # Drop a change (reparents children)
jj split --paths FILE -m "msg" # Split files out of @ into a new change
jj rebase -r <rev> -d <dest>   # Rebase a change onto a new destination

# Bookmarks (like git branches)
jj bookmark create <name>     # Create bookmark at @
jj bookmark delete <name>     # Delete bookmark
jj bookmark list               # List bookmarks

# Git interop
jj git push                    # Push bookmarks to remote
jj git fetch                   # Fetch from remote
jj git push --bookmark <name>  # Push specific bookmark

# File operations
jj file untrack <path>         # Stop tracking a file (requires .gitignore entry)
```

### Do NOT use

- `git commit`, `git add`, `git push` — use jj equivalents
- `git stash` — not needed, jj handles working copy automatically
- `git branch` — use `jj bookmark` instead
- `git rebase -i` — use `jj squash`, `jj split`, `jj rebase` individually

### Revsets

jj uses revset expressions to query changes:

```bash
jj log -r "@"                  # Working copy
jj log -r "@-"                 # Parent of working copy
jj log -r "@-+"                # Children of parent (siblings)
jj log -r "ancestors(@, 20)"   # Last 20 ancestors
jj log -r "all()"              # Everything
jj log -r "bookmarks()"        # All bookmarked changes
```

## Build

```bash
just build    # Full build (Rust FFI + Swift app)
just run      # Build and launch
just test     # Run Rust tests
just ffi      # Rebuild just the FFI/bindings
```

## Project Structure

- `crates/jayjay-core/` — Rust core (jj-lib wrapper, diff, tree-sitter)
- `crates/jayjay-uniffi/` — uniffi Swift bindings
- `crates/jayjay-cli/` — Native CLI launcher
- `shell/mac/` — macOS SwiftUI app
- `Justfile` — Build commands
- `uniffi.toml` — at `crates/jayjay-uniffi/uniffi.toml`
