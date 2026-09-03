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
            Button { showRevsetFilter.toggle() } label: {
                Label("Filter", systemImage: "line.3.horizontal.decrease.circle")
            }
            .help("Filter by revset")
            Button { viewModel.refresh() } label: {
                RefreshSpinner(animating: viewModel.isRefreshingInFlight)
            }
            .keyboardShortcut("r")
            .help("Refresh (⌘R)")
            syncButton(
                .pull,
                inFlight: viewModel.isPullingInFlight,
                start: { viewModel.gitFetch() },
                cancel: { viewModel.cancelPull() }
            )
            syncButton(
                .push,
                inFlight: viewModel.isPushingInFlight,
                start: { viewModel.gitPush(bookmark: "") },
                cancel: { viewModel.cancelPush() }
            )
        }

        repositoryTitle

        ToolbarSpacer(.flexible)

        ToolbarItemGroup(placement: .primaryAction) {
            Button { settings.openInEditor(filePath: ".", repoPath: viewModel.repoPath) } label: {
                Label("Editor", systemImage: "curlybraces")
            }
            .help("Open repository in \(settings.externalEditor.title)")
            Button { settings.openInTerminal(at: viewModel.repoPath) } label: {
                Label("Terminal", systemImage: "terminal")
            }
            .help("Open repository in \(settings.terminal.title)")
            Button { openSettings() } label: {
                Label("Settings", systemImage: "gearshape")
            }
            .help("Settings")
        }
    }

    private func syncButton(
        _ direction: SyncArrowIndicator.Direction,
        inFlight: Bool,
        start: @escaping () -> Void,
        cancel: @escaping () -> Void
    ) -> some View {
        Button {
            if inFlight {
                cancel()
            } else {
                start()
            }
        } label: {
            SyncArrowIndicator(direction: direction, animating: inFlight)
        }
        .help(inFlight ? "Cancel \(direction.label)" : direction.help)
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
}
