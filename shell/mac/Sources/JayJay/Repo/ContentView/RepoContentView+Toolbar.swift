import JayJayCore
import SwiftUI

extension RepoContentView {
    @ToolbarContentBuilder
    var toolbarContent: some ToolbarContent {
        ToolbarItem(placement: .navigation) {
            ToolbarCollapse(reveal: sidebarReveal, trailingOrigin: sidebarShownWidth + PaneLayout.dividerWidth + 12) {
                let title = settings.sidebarHidden ? Localized.Menu.showSidebar : Localized.Menu.hideSidebar
                toolbarButton(
                    .sidebarToggle,
                    help: AppShortcut.toggleSidebar.help(title),
                    action: { settings.sidebarHidden.toggle() },
                    label: { Label(title, systemImage: "sidebar.leading") }
                )
                .accessibilityIdentifier(AID.Toolbar.sidebarToggle)
            } collapsing: {
                HStack(spacing: 0) {
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
            } trailing: {
                repositoryTitle
            }
            .onGeometryChange(for: CGFloat.self) { $0.size.width } action: { toolbarRoom.leadingItemWidth = $0 }
        }
        .sharedBackgroundVisibility(.hidden)

        ToolbarItem(placement: .principal) {
            RevsetBar(
                actions: viewModel,
                bookmarks: viewModel.bookmarks,
                workspaces: viewModel.workspaces.map(\.name),
                vocabulary: viewModel.revsetVocabulary,
                editRequest: revsetEditRequest,
                room: toolbarRoom,
                pane: operationLog,
                paneWidth: operationLogWidth
            )
        }
        .sharedBackgroundVisibility(.hidden)

        ToolbarSpacer(.flexible)

        ToolbarItemGroup(placement: .primaryAction) {
            appButton(.editor, app: settings.externalEditor.title, glyph: settings.externalEditor.toolbarGlyph) {
                settings.openInEditor(filePath: ".", repoPath: viewModel.repoPath)
            }
            appButton(.terminal, app: settings.terminal.title, glyph: settings.terminal.toolbarGlyph) {
                settings.openInTerminal(at: viewModel.repoPath)
            }
        }

        ToolbarItem(placement: .primaryAction) {
            let title = showsOperationLog ? Localized.Menu.hideOperationLog : Localized.Menu.showOperationLog
            OperationLogToolbarSlot(reveal: operationLog, paneWidth: operationLogWidth, room: toolbarRoom) {
                toolbarButton(
                    .operationLog,
                    help: AppShortcut.operationLog.help(title),
                    action: { showsOperationLog.toggle() },
                    label: { Label(title, systemImage: "clock.arrow.circlepath") }
                )
                .accessibilityIdentifier(AID.Toolbar.operationLog)
                .padding(.horizontal, 1)
                .glassEffect(.regular.interactive(), in: .capsule)
            }
        }
        .sharedBackgroundVisibility(.hidden)
    }

    private func appButton(
        _ stop: KeyboardFocusStop,
        app: String,
        glyph: ToolbarAppMark.Glyph,
        action: @escaping () -> Void
    ) -> some View {
        let title = Localized.Menu.openIn(app: app)
        return toolbarButton(stop, help: title, action: action) {
            Label { Text(title) } icon: { ToolbarAppMark(glyph: glyph) }
        }
        .disabled(viewModel.repo.isRemote())
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

    private var repositoryTitle: some View {
        RepoTitlePicker(
            repoPath: viewModel.repoPath,
            repositoryName: viewModel.repositoryName,
            workspaces: viewModel.workspaces,
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
