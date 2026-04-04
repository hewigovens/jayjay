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
        items.append(CommandPaletteItem(
            title: "Toggle Hide Git LFS-backed Files",
            icon: "externaldrive",
            category: "View"
        ) { settings.hideGitLfsDiffs.toggle() })

        for (label, revset) in [
            ("Show All", "all()"),
            ("Show Mine", "mine()"),
            ("Show Bookmarks", "bookmarks()"),
            ("Show Conflicts", "conflict()"),
            ("Show Mutable", "mutable()"),
            ("Show Trunk", "trunk().."),
            ("Reset Filter", RepoViewModel.buildDefaultRevset())
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

        items.append(CommandPaletteItem(title: "Git Pull (fetch + rebase)", icon: "arrow.down.circle", category: "Git") {
            viewModel.gitFetch()
        })
        items.append(CommandPaletteItem(title: "Git Push", icon: "arrow.up.circle", category: "Git") {
            viewModel.gitPush(bookmark: "")
        })

        items.append(CommandPaletteItem(
            title: "Bookmark Manager",
            icon: "bookmark",
            category: "Repository"
        ) { showBookmarkManager = true })
        items.append(CommandPaletteItem(
            title: "Clean Up Stale Bookmarks",
            icon: "bookmark.slash",
            category: "Repository"
        ) { viewModel.forgetStaleBookmarks() })

        if let selection {
            let short = String(selection.prefix(8))
            // Safe actions — show selected change ID so user knows the target
            items.append(CommandPaletteItem(
                title: "New Child Change (\(short))",
                icon: "plus.circle",
                category: "Change"
            ) { viewModel.newChange(parent: selection) })
            items.append(CommandPaletteItem(
                title: "Edit / Switch To (\(short))",
                icon: "pencil.circle",
                category: "Change"
            ) { viewModel.edit(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Duplicate (\(short))",
                icon: "doc.on.doc",
                category: "Change"
            ) { viewModel.duplicate(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Cherry-pick / Graft (\(short))",
                icon: "doc.on.clipboard",
                category: "Change"
            ) { viewModel.graft(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Revert Change (\(short))",
                icon: "arrow.uturn.backward",
                category: "Change"
            ) { viewModel.backout(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Create Bookmark on \(short)",
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

        items.append(CommandPaletteItem(
            title: "Zoom In",
            icon: "plus.magnifyingglass",
            category: "View"
        ) { settings.fontSize = min(24, settings.fontSize + 1) })
        items.append(CommandPaletteItem(
            title: "Zoom Out",
            icon: "minus.magnifyingglass",
            category: "View"
        ) { settings.fontSize = max(9, settings.fontSize - 1) })
        items.append(CommandPaletteItem(
            title: "Reset Zoom",
            icon: "1.magnifyingglass",
            category: "View"
        ) { settings.fontSize = 12 })

        items.append(CommandPaletteItem(title: "Show in Finder", icon: "folder", category: "Tools") {
            RepositoryActions.showInFinder(repoPath: viewModel.repoPath)
        })
        items.append(CommandPaletteItem(
            title: "View Remote Repository",
            icon: "globe",
            category: "Tools"
        ) {
            if let url = RepositoryCommands.getRemoteURL(at: viewModel.repoPath) {
                RepositoryCommands.openGitURL(url)
            }
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
            icon: "arrow.uturn.backward.circle",
            category: "Repository"
        ) { showUndo() })
        items.append(CommandPaletteItem(title: "Settings", icon: "gearshape", category: "App") {
            openSettings()
        })

        commandPanel.show(items: items, repoPath: viewModel.repoPath)
    }
}
