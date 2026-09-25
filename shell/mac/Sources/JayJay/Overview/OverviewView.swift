import JayJayCore
import SwiftUI

struct OverviewView: View {
    @Bindable var viewModel: OverviewViewModel
    @Environment(RepoWindowManager.self) private var windowManager
    @Environment(AppSettings.self) private var settings
    @FocusState private var canvasFocused: Bool
    @State private var isFilterShown = false
    @State private var filterFocusGeneration = 0

    var body: some View {
        VStack(spacing: 0) {
            statsBar
            Divider()
            HStack(spacing: 0) {
                content
                if let lane = viewModel.selectedLane, let change = viewModel.selectedChange {
                    Divider()
                    OverviewChangePanel(
                        change: change,
                        files: viewModel.selectedChangeFiles,
                        onClose: { viewModel.selectedChangeId = nil },
                        onShowInGraph: { showInGraph(lane, change) }
                    )
                    .task(id: change.commitId.id) { await viewModel.loadSelectedChangeFiles() }
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                if isFilterShown {
                    filterField
                        .background(
                            Button("Filter Lanes") { filterFocusGeneration += 1 }
                                .keyboardShortcut("f")
                                .hidden()
                        )
                } else {
                    Button {
                        isFilterShown = true
                    } label: {
                        Label("Filter Lanes", systemImage: "magnifyingglass")
                    }
                    .keyboardShortcut("f")
                    .help("Filter lanes (⌘F)")
                }
            }
        }
        .alert(
            "Abandon \u{201C}\(viewModel.pendingAbandon?.title ?? "")\u{201D}?",
            isPresented: isPresented($viewModel.pendingAbandon),
            presenting: viewModel.pendingAbandon
        ) { request in
            Button("Abandon", role: .destructive) { viewModel.abandon(request) }
            Button("Cancel", role: .cancel) {}
        } message: { request in
            Text(request.message)
        }
        .alert(
            "Delete Workspace \(viewModel.pendingWorkspaceDelete?.name ?? "")?",
            isPresented: isPresented($viewModel.pendingWorkspaceDelete),
            presenting: viewModel.pendingWorkspaceDelete
        ) { workspace in
            Button("Delete", role: .destructive) { removeWorkspace(workspace, deleteFromDisk: true) }
            Button("Cancel", role: .cancel) {}
        } message: { workspace in
            Text("This closes its window, forgets the workspace, and deletes its directory from disk:\n\(workspace.path)")
        }
    }

    private func isPresented(_ request: Binding<(some Any)?>) -> Binding<Bool> {
        Binding(get: { request.wrappedValue != nil }, set: {
            if !$0 {
                request.wrappedValue = nil
            }
        })
    }

    private var filterField: some View {
        FileFilterField(
            text: $viewModel.filter,
            placeholder: "Filter lanes",
            accessibilityIdentifier: AID.Overview.filterField,
            focusGeneration: filterFocusGeneration,
            onSubmit: {
                if viewModel.filter.isEmpty {
                    isFilterShown = false
                }
                canvasFocused = true
            },
            onCancel: dismissFilter
        )
        .frame(width: 220)
    }

    private func dismissFilter() {
        viewModel.filter = ""
        isFilterShown = false
        canvasFocused = true
    }

    @ViewBuilder
    private var content: some View {
        if let snapshot = viewModel.snapshot {
            if snapshot.overview.lanes.isEmpty {
                emptyState("No mutable changes. Every change is on trunk or immutable.")
            } else {
                GeometryReader { proxy in
                    ScrollView([.horizontal, .vertical]) {
                        OverviewCanvas(
                            lanes: snapshot.overview.lanes,
                            laneIds: viewModel.laneIds,
                            groups: viewModel.visibleGroups,
                            trunkName: viewModel.trunkName,
                            selectedLaneId: $viewModel.selectedLaneId,
                            selectedChangeId: $viewModel.selectedChangeId,
                            onShowInGraph: showInGraph,
                            actions: OverviewLaneActions(
                                workspaceInfo: viewModel.workspaceInfo(named:),
                                openWorkspace: { windowManager.openRepo($0.path) },
                                rebaseOntoTrunk: viewModel.rebaseLaneOntoTrunk,
                                abandon: { viewModel.pendingAbandon = $0 },
                                forgetWorkspace: requestForget
                            )
                        )
                        .frame(minWidth: proxy.size.width, minHeight: proxy.size.height, alignment: .topLeading)
                        .contentShape(Rectangle())
                        .onTapGesture { viewModel.selectedChangeId = nil }
                    }
                }
                .background(Color(nsColor: .windowBackgroundColor))
                .focusable()
                .focusEffectDisabled()
                .focused($canvasFocused)
                // The window is not key yet when the canvas appears, so the focus request waits a turn.
                .onAppear { DispatchQueue.main.async { canvasFocused = true } }
                .onKeyPress(.leftArrow) {
                    viewModel.selectNeighbor(-1)
                    return .handled
                }
                .onKeyPress(.rightArrow) {
                    viewModel.selectNeighbor(1)
                    return .handled
                }
                .onKeyPress(.downArrow) {
                    viewModel.selectChangeNeighbor(1)
                    return .handled
                }
                .onKeyPress(.upArrow) {
                    viewModel.selectChangeNeighbor(-1)
                    return .handled
                }
                .onKeyPress(.escape) {
                    guard viewModel.selectedChangeId != nil else { return .ignored }
                    viewModel.selectedChangeId = nil
                    return .handled
                }
                .onKeyPress(.return) {
                    guard let lane = viewModel.selectedLane else { return .ignored }
                    showInGraph(lane, viewModel.selectedChange)
                    return .handled
                }
            }
        } else if let error = viewModel.error {
            emptyState(error)
        } else {
            emptyState("Loading…")
        }
    }

    private func emptyState(_ message: String) -> some View {
        Text(message)
            .jayjayFont(12)
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var statsBar: some View {
        let overview = viewModel.snapshot?.overview
        let lanes = overview?.lanes ?? []
        let changeCount = lanes.reduce(0) { $0 + $1.changes.count }
        let behind = lanes.filter(\.isBehindTrunk).count
        let attention = lanes.filter { !$0.attention.isEmpty }.count
        return HStack(spacing: 16) {
            stat(lanes.count, "lane", "lanes")
            stat(changeCount, "change", "changes")
            stat(Int(overview?.workspaceCount ?? 0), "workspace", "workspaces")
            stat(behind, "behind trunk", "behind trunk", tint: behind > 0 ? .orange : nil)
            stat(attention, "needs attention", "need attention", tint: attention > 0 ? .orange : nil)
            Spacer()
            if let error = viewModel.actionError ?? (viewModel.snapshot == nil ? nil : viewModel.error) {
                Text(error)
                    .jayjayFont(11)
                    .foregroundStyle(.red)
                    .lineLimit(1)
            }
        }
        .padding(.horizontal, 14)
        .frame(height: 30)
    }

    private func stat(_ count: Int, _ singular: String, _ plural: String, tint: Color? = nil) -> some View {
        HStack(spacing: 4) {
            Text("\(count)")
                .jayjayFont(11, weight: .semibold)
            Text(count == 1 ? singular : plural)
                .jayjayFont(11)
        }
        .foregroundStyle(tint ?? .secondary)
    }

    private func requestForget(_ workspace: WorkspaceInfo, deleteFromDisk: Bool) {
        if deleteFromDisk, !settings.skipWorkspaceDeleteConfirmation {
            viewModel.pendingWorkspaceDelete = workspace
        } else {
            removeWorkspace(workspace, deleteFromDisk: deleteFromDisk)
        }
    }

    private func removeWorkspace(_ workspace: WorkspaceInfo, deleteFromDisk: Bool) {
        guard let storePath = viewModel.repositoryStorePath else { return }
        let viewModel = viewModel
        let windowManager = windowManager
        Task { @MainActor in
            await windowManager.withWorkspaceRemoval(workspace, repositoryStorePath: storePath) {
                await viewModel.forgetWorkspace(workspace, deleteFromDisk: deleteFromDisk)
            }
        }
    }

    private func showInGraph(_ lane: OverviewLane, _ change: OverviewChange?) {
        windowManager.showInGraph(
            repoPath: viewModel.targetRepoPath(for: lane),
            headCommitId: lane.head.commitId.id,
            selecting: (change ?? lane.head).commitId.id
        )
    }
}
