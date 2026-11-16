import JayJayCore
import SwiftUI

extension RepoContentView {
    /// Lives on the sidebar column so it sits above the sidebar and hides with it.
    @ToolbarContentBuilder
    var sidebarToolbarContent: some ToolbarContent {
        ToolbarItemGroup(placement: .automatic) {
            toolbarButton(
                .refresh,
                help: viewModel.refreshMode.help,
                action: { viewModel.runRefresh() },
                label: {
                    RefreshSpinner(
                        animating: (viewModel.isRefreshingInFlight && !viewModel.isRefreshingSilently) || isSwitchingWorkspace
                    )
                    .overlay(alignment: .topTrailing) {
                        if viewModel.refreshMode.showsBadge {
                            Circle().fill(.orange).frame(width: 7, height: 7).offset(x: 3, y: -3)
                                .accessibilityIdentifier(AID.Toolbar.staleBadge)
                        }
                    }
                }
            )
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
    }

    @ToolbarContentBuilder
    var toolbarContent: some ToolbarContent {
        repositoryTitle

        ToolbarItem(placement: .principal) {
            RevsetBar(
                actions: viewModel,
                bookmarks: viewModel.bookmarks,
                workspaces: viewModel.workspaces.map(\.name),
                vocabulary: viewModel.revsetVocabulary,
                editRequest: revsetEditRequest
            )
        }
        .sharedBackgroundVisibility(.hidden)

        ToolbarSpacer(.flexible)

        ToolbarItemGroup(placement: .primaryAction) {
            if let pr = viewModel.prInfo, let url = URL(string: pr.url) {
                Link(destination: url) {
                    Label("#\(pr.number) \(pr.stateLabel)", systemImage: pr.checksIcon)
                }
                .help(pr.title)
            }
            if sshMachine(location: viewModel.repoPath) != nil, !viewModel.sshConnected {
                Button(Localized.Ssh.reconnect, systemImage: "arrow.clockwise") { viewModel.reconnectSSH() }
                    .help(viewModel.sshErrorMessage ?? viewModel.sshStatusLabel)
            }
            Button(Localized.Menu.operationLogEllipsis, systemImage: "clock.arrow.trianglehead.counterclockwise.rotate.90") {
                if showsOperationLog {
                    showsOperationLog = false
                } else {
                    showOperationLog()
                }
            }
            .help(viewModel.currentOperationDescription.isEmpty
                ? AppShortcut.operationLog.help(Localized.Menu.operationLogEllipsis)
                : "\(AppShortcut.operationLog.help(Localized.Menu.operationLogEllipsis))\n\(viewModel.currentOperationDescription)")
            toolbarButton(
                .editor,
                help: Localized.Repo.openRepositoryIn(app: settings.externalEditor.title),
                action: { settings.openInEditor(filePath: ".", repoPath: viewModel.repoPath) },
                label: { Label { Text(Localized.Settings.editor) } icon: { settings.externalEditor.toolbarIcon } }
            )
            .disabled(viewModel.repo.isRemote())
            toolbarButton(
                .terminal,
                help: Localized.Repo.openRepositoryIn(app: settings.terminal.title),
                action: { settings.openInTerminal(at: viewModel.repoPath) },
                label: { Label(Localized.Settings.terminal, systemImage: "terminal") }
            )
            .disabled(viewModel.repo.isRemote())
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
            help: inFlight ? Localized.Repo.cancelDirection(direction: direction.label) : direction.help,
            action: toggle,
            label: { SyncArrowIndicator(direction: direction, animating: inFlight) }
        )
        .disabled(!viewModel.sshConnected && !inFlight)
        .accessibilityIdentifier(direction.accessibilityIdentifier)
    }

    private var titleContext: String? {
        let parts = [sshContext, activeBookmarkSync].compactMap(\.self)
        return parts.isEmpty ? nil : parts.joined(separator: " · ")
    }

    /// The nearest tracked bookmark at or below the working copy, shown as quiet title context.
    private var activeBookmarkSync: String? {
        let changes = viewModel.changes
        guard let wcIndex = changes.firstIndex(where: { $0.isWorkingCopy }) else { return nil }
        for change in changes[wcIndex...] {
            for name in change.bookmarks {
                guard let bookmark = viewModel.bookmarks.first(where: { $0.name == name }),
                      let sync = bookmarkSync(bookmark: bookmark)
                else { continue }
                return sync.text
            }
        }
        return nil
    }

    private var repositoryTitle: some ToolbarContent {
        ToolbarItem(placement: .navigation) {
            RepoTitlePicker(
                repoPath: viewModel.repoPath,
                repositoryName: viewModel.repositoryName,
                workspaces: viewModel.workspaces,
                context: titleContext,
                onFilterWorkspace: { workspace in
                    viewModel.filterByWorkspace(name: workspace.name)
                },
                onSwitchWorkspace: { workspace in
                    guard workspace.isPathResolved else { return }
                    windowManager.switchRepo(from: viewModel, to: workspace.path, changePath: onSwitchWorkspace)
                },
                onOpenWorkspaceInNewWindow: { workspace in
                    guard workspace.isPathResolved else { return }
                    windowManager.openRepo(workspace.path)
                },
                onForget: { workspace in
                    removeWorkspace(workspace, deleteFromDisk: false)
                },
                onForgetDelete: { workspace in
                    guard workspace.isPathResolved else { return }
                    requestWorkspaceDelete(workspace)
                },
                onCreateWorkspace: { modal = .workspaceCreate },
                onOpenOverview: { windowManager.openOverview(for: viewModel.repoPath) }
            )
        }
        .sharedBackgroundVisibility(.hidden)
    }
}

private extension SyncArrowIndicator.Direction {
    var help: String {
        switch self {
            case .pull: Localized.Repo.gitPullFetchRebase
            case .push: Localized.Repo.gitPush
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
