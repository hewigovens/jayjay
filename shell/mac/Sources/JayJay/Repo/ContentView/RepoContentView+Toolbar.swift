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
                ZStack(alignment: .topTrailing) {
                    RefreshSpinner(animating: viewModel.isRefreshingInFlight)
                    if viewModel.hasWorkingCopyChanges {
                        Circle().fill(.orange).frame(width: 6, height: 6).offset(x: 2, y: -2)
                    }
                }
            }
            .keyboardShortcut("r")
            .help(viewModel.hasWorkingCopyChanges ? "Files changed — click to refresh (⌘R)" : "Refresh (⌘R)")
            Button { viewModel.gitFetch() } label: {
                SyncArrowIndicator(direction: .pull, animating: viewModel.isPullingInFlight)
            }
            .help("Git Pull (fetch + rebase)")
            Button { viewModel.gitPush(bookmark: "") } label: {
                SyncArrowIndicator(direction: .push, animating: viewModel.isPushingInFlight)
            }
            .help("Git Push")
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
                    let settings = settings
                    let viewModel = viewModel
                    let windowManager = windowManager
                    Task { @MainActor in
                        let hasRecordedPath = !workspace.path.isEmpty
                        if hasRecordedPath {
                            await windowManager.closeRepoWindowForWorkspaceRemoval(at: workspace.path)
                        }
                        defer {
                            if hasRecordedPath {
                                windowManager.finishWorkspaceRemoval(at: workspace.path)
                            }
                        }
                        do {
                            let warning: String?
                            if workspace.isPathResolved {
                                let operation = try await viewModel.workspaceRemovalGuard(
                                    name: workspace.name,
                                    expectedRoot: workspace.path,
                                    expectedOperation: workspace.operationId
                                )
                                warning = try await viewModel.workspaceForget(
                                    name: workspace.name,
                                    expectedRoot: workspace.path,
                                    expectedOperation: operation
                                )
                            } else {
                                warning = try await viewModel.workspaceForgetUnresolved(
                                    name: workspace.name,
                                    expectedOperation: workspace.operationId
                                )
                            }
                            if hasRecordedPath {
                                settings.removeRecentRepo(workspace.path)
                            }
                            if let warning {
                                viewModel.error = warning
                            }
                        } catch {
                            viewModel.present(error: error)
                        }
                    }
                },
                onForgetDelete: { workspace in
                    guard workspace.isPathResolved else { return }
                    modal = .confirmWorkspaceDelete(
                        name: workspace.name,
                        path: workspace.path,
                        operationId: workspace.operationId
                    )
                },
                onCreateWorkspace: { modal = .workspaceCreate }
            )
        }
        .sharedBackgroundVisibility(.hidden)
    }
}

struct SidebarDivider: View {
    @Binding var position: CGFloat
    let range: ClosedRange<CGFloat>
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Rectangle()
            .fill(Color.primary.opacity(0.08))
            .frame(width: 1)
            .contentShape(Rectangle().inset(by: -3))
            .onHover {
                if $0 {
                    NSCursor.resizeLeftRight.push()
                } else {
                    NSCursor.pop()
                }
            }
            .gesture(
                DragGesture(minimumDistance: 1)
                    .onChanged { position = min(max(position + $0.translation.width, range.lowerBound), range.upperBound) }
                    .onEnded { _ in settings.sidebarWidth = position }
            )
    }
}
