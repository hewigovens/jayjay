import SwiftUI

extension RepoContentView {
    func showCommandPalette() {
        var items: [CommandPaletteItem] = []
        let selection = viewModel.selectedChangeId

        items.append(CommandPaletteItem(
            title: "Refresh", icon: "arrow.triangle.2.circlepath", category: "View", shortcut: "⌘R"
        ) {
            viewModel.refresh()
        })
        items.append(CommandPaletteItem(
            title: "Toggle Side-by-Side Diff",
            icon: "rectangle.split.2x1",
            category: "View",
            keywords: ["diff", "split", "side", "by", "unified"]
        ) { settings.sideBySideDiff.toggle() })
        items.append(CommandPaletteItem(
            title: "Toggle Tree File List",
            icon: "list.bullet.indent",
            category: "View",
            keywords: ["tree", "file", "folder", "list"]
        ) { settings.treeFileList.toggle() })
        items.append(CommandPaletteItem(
            title: "Toggle Ignore Whitespace",
            icon: "text.alignleft",
            category: "View",
            keywords: ["whitespace", "diff", "ignore"]
        ) { settings.ignoreWhitespace.toggle() })
        items.append(CommandPaletteItem(
            title: "Toggle Hide Git LFS-backed Files",
            icon: "externaldrive",
            category: "View"
        ) { settings.hideGitLfsDiffs.toggle() })

        for (mode, label, icon) in [
            (AppSettings.AppearanceMode.system, "System", "circle.lefthalf.filled"),
            (.light, "Light", "sun.max"),
            (.dark, "Dark", "moon")
        ] {
            items.append(CommandPaletteItem(
                title: "Theme: \(label)",
                icon: icon,
                category: "View",
                keywords: ["theme", "appearance", "mode", "color", "scheme"]
            ) { settings.appearanceMode = mode })
        }

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

        items
            .append(CommandPaletteItem(title: "Git Pull (fetch + rebase)", icon: "arrow.down.circle", category: "Git") {
                viewModel.gitFetch()
            })
        items.append(CommandPaletteItem(title: "Git Push", icon: "arrow.up.circle", category: "Git") {
            viewModel.gitPush(bookmark: "")
        })

        items.append(CommandPaletteItem(
            title: "Bookmark Manager",
            icon: "bookmark",
            category: "Repository",
            shortcut: "⇧⌘B"
        ) { modal = .bookmarkManager })
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
            ) { viewModel.revertChange(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Create Bookmark on \(short)",
                icon: "bookmark",
                category: "Change"
            ) {
                presentBookmarkCreate(rev: selection)
            })
        }

        items.append(CommandPaletteItem(
            title: "New Workspace",
            icon: "square.on.square",
            category: "Workspace"
        ) { modal = .workspaceCreate })
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
            category: "View",
            shortcut: "⌘+"
        ) { settings.fontSize = min(24, settings.fontSize + 1) })
        items.append(CommandPaletteItem(
            title: "Zoom Out",
            icon: "minus.magnifyingglass",
            category: "View",
            shortcut: "⌘−"
        ) { settings.fontSize = max(9, settings.fontSize - 1) })
        items.append(CommandPaletteItem(
            title: "Reset Zoom",
            icon: "1.magnifyingglass",
            category: "View",
            shortcut: "⌘0"
        ) { settings.fontSize = 12 })

        items.append(CommandPaletteItem(
            title: "Show in Finder", icon: "folder", category: "Tools", shortcut: "⌥⌘F"
        ) {
            RepositoryActions.showInFinder(repoPath: viewModel.repoPath)
        })
        items.append(CommandPaletteItem(
            title: "View Remote Repository",
            icon: "globe",
            category: "Tools"
        ) {
            RepositoryCommands.openRemoteRepository(repo: viewModel.repo)
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
            title: "Undo Last Operation",
            icon: "arrow.uturn.backward.circle",
            category: "Repository",
            shortcut: "⇧⌘U"
        ) { showUndo() })
        items.append(CommandPaletteItem(
            title: "Settings", icon: "gearshape", category: "App", shortcut: "⌘,"
        ) {
            openSettings()
        })

        for feature in HelpFeatureIndex.bundled {
            items.append(CommandPaletteItem(
                title: feature.commandPaletteTitle,
                icon: "questionmark.circle",
                category: "Help",
                detail: feature.summary,
                keywords: feature.commandPaletteKeywords,
                shortcut: feature.shortcut
            ) {
                HelpBook.open(anchor: feature.helpAnchor)
            })
        }
        items.append(CommandPaletteItem(
            title: "Open JayJay User Guide",
            icon: "book",
            category: "Help",
            detail: "Open the full web guide in your browser.",
            keywords: ["help", "guide", "manual", "documentation", "docs"]
        ) {
            HelpBook.openOnlineGuide()
        })

        // Searchable keybind cheatsheet — info-only rows for keys that aren't commands (issue #87).
        items.append(.keybind(
            title: "Command Palette", icon: "command", shortcut: "⇧⌘P",
            keywords: ["palette", "command", "search"]
        ))
        items.append(.keybind(
            title: "Next / Previous Change", icon: "arrow.up.arrow.down", shortcut: "J / K",
            keywords: ["move", "select", "next", "previous", "down", "up", "navigate", "ctrl", "n", "p"]
        ))
        items.append(.keybind(
            title: "Mark File Reviewed", icon: "checkmark.circle", shortcut: "Space",
            keywords: ["review", "reviewed", "check", "diff"]
        ))
        items.append(.keybind(
            title: "Find in Diff", icon: "magnifyingglass", shortcut: "⌘F",
            keywords: ["find", "search", "diff"]
        ))
        items.append(.keybind(
            title: "Open Repository", icon: "folder.badge.plus", shortcut: "⌘O",
            keywords: ["open", "repository", "repo"]
        ))
        items.append(.keybind(
            title: "Keyboard Shortcuts", icon: "keyboard", shortcut: "⌘/",
            keywords: ["shortcut", "shortcuts", "keys", "cheatsheet", "keybind", "help"]
        ))

        commandPanel.show(
            items: items,
            repoPath: viewModel.repoPath,
            onJjCommandFinished: { result in
                guard result.exitCode == 0 else { return }
                viewModel.refresh()
            }
        )
    }
}
