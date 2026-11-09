import JayJayCore
import SwiftUI

extension RepoContentView {
    func showCommandPalette() {
        var items = viewPaletteItems + filterPaletteItems + gitPaletteItems + repositoryPaletteItems
        if let change = viewModel.selectedChangeId.flatMap(viewModel.change(for:)) {
            items += changePaletteItems(change: change)
        }
        items += multiSelectionPaletteItems
        items += workspacePaletteItems + zoomPaletteItems + toolsPaletteItems + appPaletteItems
        items += helpPaletteItems + keybindPaletteItems
        commandPanel.show(
            items: items,
            runJjCommand: { [weak viewModel] command in
                guard let viewModel else { throw CancellationError() }
                return try await viewModel.runJjCommand(command)
            },
            onJjCommandFinished: { [weak viewModel] result in
                guard result.exitCode == 0, let viewModel else { return }
                viewModel.refresh()
            }
        )
    }

    private var viewPaletteItems: [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        items.append(CommandPaletteItem(
            title: Localized.Repo.refresh, icon: "arrow.triangle.2.circlepath", category: Localized.Menu.view, shortcut: AppShortcut.refresh.symbol
        ) {
            performShortcut(.refresh)
        })
        items.append(CommandPaletteItem(
            title: Localized.Palette.bisectEllipsis,
            icon: "arrow.left.and.right",
            category: Localized.Menu.view,
            keywords: ["bisect", "binary", "search", "bug", "regression"],
            shortcut: AppShortcut.bisect.symbol
        ) {
            performShortcut(.bisect)
        })
        items.append(CommandPaletteItem(
            title: settings.sidebarHidden ? Localized.Menu.showSidebar : Localized.Menu.hideSidebar,
            icon: "sidebar.left",
            category: Localized.Menu.view,
            keywords: ["toggle", "sidebar", "dag", "history", "panel"],
            shortcut: AppShortcut.toggleSidebar.symbol
        ) { settings.sidebarHidden.toggle() })
        items.append(CommandPaletteItem(
            title: Localized.Palette.toggleSideBySideDiffTitle,
            icon: "rectangle.split.2x1",
            category: Localized.Menu.view,
            keywords: ["diff", "split", "side", "by", "unified"],
            shortcut: AppShortcut.toggleSideBySide.symbol
        ) { performShortcut(.toggleSideBySide) })
        items.append(CommandPaletteItem(
            title: Localized.Palette.toggleTreeFileList,
            icon: "list.bullet.indent",
            category: Localized.Menu.view,
            keywords: ["tree", "file", "folder", "list"],
            shortcut: AppShortcut.toggleTreeFileList.symbol
        ) { performShortcut(.toggleTreeFileList) })
        items.append(CommandPaletteItem(
            title: Localized.Palette.toggleAutoExpandDescriptions,
            icon: "arrow.up.and.line.horizontal.and.arrow.down",
            category: Localized.Menu.view,
            keywords: ["description", "commit", "message", "expand", "auto"]
        ) { settings.autoExpandDescription.toggle() })
        items.append(CommandPaletteItem(
            title: Localized.Palette.expandAllUnmodifiedLines,
            icon: "arrow.up.and.down.text.horizontal",
            category: Localized.Menu.view,
            keywords: ["context", "expand", "unmodified", "collapsed", "show", "diff"]
        ) { diffCommands.expandAllContext() })
        items.append(CommandPaletteItem(
            title: Localized.Palette.toggleIgnoreWhitespace,
            icon: "text.alignleft",
            category: Localized.Menu.view,
            keywords: ["whitespace", "diff", "ignore"],
            shortcut: AppShortcut.toggleIgnoreWhitespace.symbol
        ) { performShortcut(.toggleIgnoreWhitespace) })
        items.append(CommandPaletteItem(
            title: Localized.Palette.toggleHideGitLfsBackedFiles,
            icon: "externaldrive",
            category: Localized.Menu.view
        ) { settings.hideGitLfsDiffs.toggle() })

        for (mode, label, icon) in [
            (AppSettings.AppearanceMode.system, Localized.Settings.appearanceSystem, "circle.lefthalf.filled"),
            (.light, Localized.Settings.appearanceLight, "sun.max"),
            (.dark, Localized.Settings.appearanceDark, "moon")
        ] {
            items.append(CommandPaletteItem(
                title: Localized.Palette.theme(theme: label),
                icon: icon,
                category: Localized.Menu.view,
                keywords: ["theme", "appearance", "mode", "color", "scheme"]
            ) { settings.appearanceMode = mode })
        }
        return items
    }

    private var filterPaletteItems: [CommandPaletteItem] {
        var items = [CommandPaletteItem(
            title: Localized.Menu.filterByRevsetEllipsis,
            icon: "line.3.horizontal.decrease",
            category: Localized.Repo.filter,
            keywords: ["revset", "filter", "query"],
            shortcut: AppShortcut.revsetFilter.symbol
        ) { revsetEditRequest += 1 }]
        let presetFilters = RevsetFilterPresets.all.map { (Localized.Palette.showFilter(filter: $0.label), $0.revset) }
        for (label, revset) in presetFilters + [
            (Localized.Palette.showMutable, "mutable()"),
            (Localized.Palette.resetFilter, viewModel.defaultRevsetPreset.revset)
        ] {
            items.append(CommandPaletteItem(
                title: label,
                icon: "line.3.horizontal.decrease.circle",
                category: Localized.Repo.filter
            ) {
                viewModel.applyFilter(revset)
            })
        }
        return items
    }

    private var gitPaletteItems: [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        items
            .append(CommandPaletteItem(
                title: Localized.Menu.pull,
                icon: "arrow.down.circle",
                category: Localized.Palette.git,
                keywords: ["git", "fetch", "rebase", "sync"],
                shortcut: AppShortcut.fetch.symbol
            ) {
                performShortcut(.fetch)
            })
        items.append(CommandPaletteItem(title: Localized.Repo.gitPush, icon: "arrow.up.circle", category: Localized.Palette.git, shortcut: AppShortcut.push.symbol) {
            performShortcut(.push)
        })
        items.append(contentsOf: cancelSyncPaletteItems)
        return items
    }

    private var repositoryPaletteItems: [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        items.append(CommandPaletteItem(
            title: Localized.Menu.bookmarkManager,
            icon: "bookmark",
            category: Localized.Menu.repository,
            shortcut: AppShortcut.bookmarkManager.symbol
        ) { modal = .bookmarkManager })
        items.append(CommandPaletteItem(
            title: Localized.Menu.repoOverview,
            icon: "rectangle.split.3x1",
            category: Localized.Menu.repository,
            shortcut: AppShortcut.repoOverview.symbol
        ) { windowManager.openOverview(for: viewModel.repoPath) })
        items.append(CommandPaletteItem(
            title: Localized.Palette.cleanUpStaleBookmarks,
            icon: "bookmark.slash",
            category: Localized.Menu.repository,
            shortcut: AppShortcut.cleanUpStaleBookmarks.symbol
        ) { performShortcut(.cleanUpStaleBookmarks) })
        items.append(CommandPaletteItem(
            title: Localized.Repo.repositoryList,
            icon: "list.bullet.rectangle",
            category: Localized.Menu.repository
        ) { windowManager.showRepoList() })
        items.append(CommandPaletteItem(
            title: Localized.Ssh.openRemoteRepositoryEllipsis,
            icon: "network",
            category: Localized.Menu.repository
        ) { windowManager.openRemoteRepository() })
        return items
    }

    private var workspacePaletteItems: [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        items.append(CommandPaletteItem(
            title: Localized.Repo.newWorkspace,
            icon: "folder.badge.plus",
            category: Localized.Repo.workspace,
            shortcut: AppShortcut.newWorkspace.symbol
        ) { modal = .workspaceCreate })
        items.append(CommandPaletteItem(
            title: "\(PullRequestImportSheet.title)…",
            icon: "arrow.triangle.pull",
            category: Localized.Repo.workspace,
            keywords: ["pr", "github", "import", "checkout", "merge request"],
            shortcut: AppShortcut.importPullRequest.symbol
        ) { modal = .pullRequestImport })
        for workspace in viewModel.workspaces where !workspace.isCurrent {
            if workspace.isPathResolved {
                items.append(CommandPaletteItem(
                    title: Localized.Palette.switchTo(name: workspace.name),
                    icon: "arrow.right.square",
                    category: Localized.Repo.workspace
                ) {
                    windowManager.switchRepo(from: viewModel, to: workspace.path, changePath: onSwitchWorkspace)
                })
            }
            items.append(CommandPaletteItem(
                title: Localized.Palette.forgetWorkspaceNamed(name: workspace.name),
                icon: "folder.badge.minus",
                category: Localized.Repo.workspace
            ) { removeWorkspace(workspace, deleteFromDisk: false) })
            if workspace.isPathResolved {
                items.append(CommandPaletteItem(
                    title: Localized.Palette.forgetDeleteWorkspaceNamed(name: workspace.name),
                    icon: "trash",
                    category: Localized.Repo.workspace
                ) { requestWorkspaceDelete(workspace) })
            }
        }
        return items
    }

    private var zoomPaletteItems: [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        items.append(CommandPaletteItem(
            title: Localized.Menu.zoomIn,
            icon: "plus.magnifyingglass",
            category: Localized.Menu.view,
            shortcut: AppShortcut.zoomIn.symbol
        ) { settings.zoomIn() })
        items.append(CommandPaletteItem(
            title: Localized.Menu.zoomOut,
            icon: "minus.magnifyingglass",
            category: Localized.Menu.view,
            shortcut: AppShortcut.zoomOut.symbol
        ) { settings.zoomOut() })
        items.append(CommandPaletteItem(
            title: Localized.Menu.resetZoom,
            icon: "1.magnifyingglass",
            category: Localized.Menu.view,
            shortcut: AppShortcut.resetZoom.symbol
        ) { settings.resetZoom() })
        return items
    }

    private var toolsPaletteItems: [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        if !viewModel.repoPath.isSSHLocation {
            items.append(CommandPaletteItem(
                title: Localized.Common.showIn(app: "Finder"), icon: "folder", category: Localized.Palette.tools, shortcut: AppShortcut.showInFinder.symbol
            ) {
                RepositoryActions.showInFinder(repoPath: viewModel.repoPath)
            })
        }
        if viewModel.hasWebRemote {
            items.append(CommandPaletteItem(
                title: Localized.Menu.viewRemoteRepository,
                icon: "globe",
                category: Localized.Palette.tools
            ) {
                RepositoryCommands.openRemoteRepository(repo: viewModel.repo)
            })
        }
        items.append(CommandPaletteItem(
            title: Localized.Menu.openIn(app: settings.externalEditor.title),
            icon: "curlybraces",
            category: Localized.Palette.tools
        ) { settings.openInEditor(filePath: ".", repoPath: viewModel.repoPath) })
        items.append(CommandPaletteItem(
            title: Localized.Menu.openIn(app: settings.terminal.title),
            icon: "terminal",
            category: Localized.Palette.tools
        ) { settings.openInTerminal(at: viewModel.repoPath) })
        return items
    }

    private var appPaletteItems: [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        items.append(CommandPaletteItem(
            title: Localized.Menu.operationLogEllipsis,
            icon: "clock.arrow.circlepath",
            category: Localized.Menu.repository,
            shortcut: AppShortcut.operationLog.symbol
        ) { showOperationLog() })
        items.append(CommandPaletteItem(
            title: Localized.Repo.settings, icon: "gearshape", category: Localized.Palette.app, shortcut: AppShortcut.settings.symbol
        ) {
            openSettings()
        })
        return items
    }

    private var helpPaletteItems: [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        for feature in HelpFeatureIndex.bundled {
            items.append(CommandPaletteItem(
                title: feature.commandPaletteTitle,
                icon: "questionmark.circle",
                category: Localized.Menu.help,
                detail: feature.summary,
                keywords: feature.commandPaletteKeywords,
                shortcut: feature.effectiveShortcut(in: ShortcutStore.shared)
            ) {
                HelpBook.open(anchor: feature.helpAnchor)
            })
        }
        items.append(CommandPaletteItem(
            title: Localized.Palette.openJayjayUserGuide,
            icon: "book",
            category: Localized.Menu.help,
            detail: Localized.Palette.openTheFullWebGuideIn,
            keywords: ["help", "guide", "manual", "documentation", "docs"]
        ) {
            HelpBook.openOnlineGuide()
        })
        items.append(CommandPaletteItem(
            title: Localized.Menu.sendFeedback,
            icon: "envelope",
            category: Localized.Menu.help,
            keywords: ["email", "contact", "support"]
        ) {
            FeedbackEmail.open()
        })
        return items
    }

    private var keybindPaletteItems: [CommandPaletteItem] {
        ShortcutStore.shared.bindings.filter { !$0.chords.isEmpty }.map { binding in
            .keybind(title: binding.title, shortcut: binding.chords.map { $0.joined() }.joined(separator: " / "))
        }
    }

    private var cancelSyncPaletteItems: [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        if viewModel.isPullingInFlight {
            items.append(CommandPaletteItem(title: Localized.Palette.cancelPull, icon: "xmark.circle", category: Localized.Palette.git) {
                viewModel.cancelPull()
            })
        }
        if viewModel.isPushingInFlight {
            items.append(CommandPaletteItem(title: Localized.Palette.cancelPush, icon: "xmark.circle", category: Localized.Palette.git) {
                viewModel.cancelPush()
            })
        }
        return items
    }
}
