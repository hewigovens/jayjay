import SwiftUI

extension RepoContentView {
    @ToolbarContentBuilder
    var toolbarContent: some ToolbarContent {
        ToolbarItemGroup(placement: .navigation) {
            BookmarkPicker(
                bookmarks: viewModel.bookmarks,
                actions: viewModel,
                onSelect: {
                    revsetDraft = $0
                    applyRevset()
                }
            )
            toolbarButton(
                .revsetFilter,
                help: "Filter by revset",
                action: {
                    showRevsetFilter.toggle()
                    if showRevsetFilter {
                        keyboardFocus.updateInputFocus(.revsetInput, isFocused: true)
                    }
                },
                label: { Label("Filter", systemImage: "line.3.horizontal.decrease.circle") }
            )
            toolbarButton(
                .refresh,
                help: "Refresh (⌘R)",
                action: { viewModel.refresh() },
                label: { RefreshSpinner(animating: viewModel.isRefreshingInFlight) }
            )
            .keyboardShortcut("r")
            syncButton(.pull, inFlight: viewModel.isPullingInFlight) {
                if viewModel.isPullingInFlight {
                    viewModel.cancelPull()
                } else {
                    viewModel.gitFetch()
                }
            }
            syncButton(.push, inFlight: viewModel.isPushingInFlight) {
                if viewModel.isPushingInFlight {
                    viewModel.cancelPush()
                } else {
                    viewModel.gitPush(bookmark: "")
                }
            }
        }

        repositoryTitle

        ToolbarSpacer(.flexible)

        ToolbarItemGroup(placement: .primaryAction) {
            toolbarButton(
                .editor,
                help: "Open repository in \(settings.externalEditor.title)",
                action: { settings.openInEditor(filePath: ".", repoPath: viewModel.repoPath) },
                label: { Label("Editor", systemImage: "curlybraces") }
            )
            toolbarButton(
                .terminal,
                help: "Open repository in \(settings.terminal.title)",
                action: { settings.openInTerminal(at: viewModel.repoPath) },
                label: { Label("Terminal", systemImage: "terminal") }
            )
            toolbarButton(
                .settings,
                help: "Settings",
                action: { openSettings() },
                label: { Label("Settings", systemImage: "gearshape") }
            )
        }
    }

    /// The label carries the Tab stop so the focus ring draws inside the toolbar item.
    private func toolbarButton(
        _ stop: KeyboardFocusStop,
        help: String,
        action: @escaping () -> Void,
        @ViewBuilder label: () -> some View
    ) -> some View {
        Button(action: action) {
            label().keyboardFocusStop(stop, action: action)
        }
        .help(help)
    }

    private func syncButton(
        _ direction: SyncArrowIndicator.Direction,
        inFlight: Bool,
        toggle: @escaping () -> Void
    ) -> some View {
        toolbarButton(
            direction.focusStop,
            help: inFlight ? "Cancel \(direction.label)" : direction.help,
            action: toggle,
            label: { SyncArrowIndicator(direction: direction, animating: inFlight) }
        )
        .accessibilityIdentifier(direction.accessibilityIdentifier)
    }

    private var repositoryTitle: some ToolbarContent {
        ToolbarItem(placement: .navigation) {
            RepoTitlePicker(
                repoPath: viewModel.repoPath,
                workspaces: viewModel.workspaces,
                onOpenWorkspace: { workspace in
                    guard workspace.isPathResolved else { return }
                    windowManager.openRepo(workspace.path)
                },
                onForget: { workspace in
                    removeWorkspace(workspace, deleteFromDisk: false)
                },
                onForgetDelete: { workspace in
                    guard workspace.isPathResolved else { return }
                    modal = .confirmWorkspaceDelete(workspace: workspace)
                },
                onCreateWorkspace: { modal = .workspaceCreate },
                onRefresh: { viewModel.refreshWorkspaces() }
            )
        }
        .sharedBackgroundVisibility(.hidden)
    }
}

private extension SyncArrowIndicator.Direction {
    var help: String {
        switch self {
            case .pull: "Git Pull (fetch + rebase)"
            case .push: "Git Push"
        }
    }

    var accessibilityIdentifier: String {
        switch self {
            case .pull: AID.Toolbar.pull
            case .push: AID.Toolbar.push
        }
    }

    var focusStop: KeyboardFocusStop {
        switch self {
            case .pull: .pull
            case .push: .push
        }
    }
}
