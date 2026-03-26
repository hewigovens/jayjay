import SwiftUI

extension RepoContentView {
    func showCommandPalette() {
        var items: [CommandPaletteItem] = []
        let selection = viewModel.selectedChangeId

        items.append(CommandPaletteItem(title: "Refresh", icon: "arrow.triangle.2.circlepath", category: "View") {
            viewModel.refresh()
        })
        items.append(CommandPaletteItem(
            title: "Toggle Side-by-Side Diff",
            icon: "rectangle.split.2x1",
            category: "View"
        ) { settings.sideBySideDiff.toggle() })
        items.append(CommandPaletteItem(
            title: "Toggle Tree View",
            icon: "list.bullet.indent",
            category: "View"
        ) { settings.treeFileList.toggle() })
        items.append(CommandPaletteItem(
            title: "Toggle Ignore Whitespace",
            icon: "text.alignleft",
            category: "View"
        ) { settings.ignoreWhitespace.toggle() })

        for (label, revset) in [
            ("Show All", "all()"),
            ("Show Mine", "mine()"),
            ("Show Bookmarks", "bookmarks()"),
            ("Show Conflicts", "conflict()")
        ] {
            items.append(CommandPaletteItem(
                title: label,
                icon: "line.3.horizontal.decrease.circle",
                category: "Filter"
            ) {
                revsetDraft = revset
                applyRevset()
            })
        }

        items.append(CommandPaletteItem(title: "Git Pull", icon: "arrow.down.circle", category: "Git") {
            viewModel.gitFetch()
        })
        items.append(CommandPaletteItem(title: "Git Push", icon: "arrow.up.circle", category: "Git") {
            viewModel.gitPush(bookmark: "")
        })

        if let selection {
            items.append(CommandPaletteItem(
                title: "New Child Change",
                icon: "plus.circle",
                category: "Change"
            ) { viewModel.newChange(parent: selection) })
            items.append(CommandPaletteItem(
                title: "Edit (Switch To)",
                icon: "pencil.circle",
                category: "Change"
            ) { viewModel.edit(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Squash into Parent",
                icon: "arrow.down.left.circle",
                category: "Change"
            ) { viewModel.squash(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Duplicate",
                icon: "doc.on.doc",
                category: "Change"
            ) { viewModel.duplicate(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Cherry-pick (Graft)",
                icon: "arrow.triangle.branch",
                category: "Change"
            ) { viewModel.graft(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Absorb into Ancestors",
                icon: "arrow.merge",
                category: "Change"
            ) { viewModel.absorb(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Revert Change",
                icon: "arrow.uturn.backward.circle",
                category: "Change"
            ) { viewModel.backout(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Abandon",
                icon: "trash",
                category: "Change"
            ) { requestAbandon(selection) })
            items.append(CommandPaletteItem(
                title: "Create Bookmark Here",
                icon: "bookmark",
                category: "Change"
            ) {
                bookmarkCreateRev = selection
                bookmarkCreateName = ""
            })
        }

        items.append(CommandPaletteItem(
            title: "New Workspace",
            icon: "square.on.square",
            category: "Workspace"
        ) { showWorkspaceCreate = true })
        for workspace in viewModel.workspaceList() where !workspace.isCurrent {
            items.append(CommandPaletteItem(
                title: "Switch to \(workspace.name)",
                icon: "arrow.right.square",
                category: "Workspace"
            ) { windowManager.openRepo(workspace.path) })
        }

        items.append(CommandPaletteItem(title: "Show in Finder", icon: "folder", category: "Tools") {
            RepositoryActions.showInFinder(repoPath: viewModel.repoPath)
        })
        items.append(CommandPaletteItem(
            title: "Open in \(settings.externalEditor.title)",
            icon: "curlybraces",
            category: "Tools"
        ) { settings.openInEditor(filePath: ".", repoPath: viewModel.repoPath) })
        items.append(CommandPaletteItem(
            title: "Open in \(settings.terminal.title)",
            icon: "terminal",
            category: "Tools"
        ) { settings.openInTerminal(at: viewModel.repoPath) })

        items.append(CommandPaletteItem(
            title: "Undo (Operation Log)",
            icon: "arrow.uturn.backward",
            category: "Repository"
        ) { showUndo() })
        items.append(CommandPaletteItem(title: "Settings", icon: "gearshape", category: "App") {
            openSettings()
        })

        commandPanel.show(items: items, repoPath: viewModel.repoPath)
    }
}
