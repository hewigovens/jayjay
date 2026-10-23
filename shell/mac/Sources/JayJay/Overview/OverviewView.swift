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
                } else if viewModel.isLanePanelShown, let lane = viewModel.selectedLane {
                    Divider()
                    OverviewLanePanel(
                        lane: lane,
                        trunkName: viewModel.trunkName,
                        workspaceInfo: viewModel.workspaceInfo(named:),
                        onClose: { viewModel.isLanePanelShown = false },
                        onShowInGraph: { showInGraph(lane, nil) },
                        onRebase: { viewModel.rebaseLaneOntoTrunk(lane) },
                        onOpenWorkspace: { windowManager.openRepo($0.path) },
                        onSelectChange: { viewModel.selectedChangeId = $0.commitId.id }
                    )
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                if isFilterShown {
                    filterField
                        .background(
                            Button(Localized.Overview.filterLanesTitle) { filterFocusGeneration += 1 }
                                .keyboardShortcut(AppShortcut.filterOverviewLanes)
                                .hidden()
                        )
                } else {
                    Button {
                        isFilterShown = true
                    } label: {
                        Label(Localized.Overview.filterLanesTitle, systemImage: "magnifyingglass")
                    }
                    .keyboardShortcut(AppShortcut.filterOverviewLanes)
                    .help(Localized.Overview.filterLanesShortcut(shortcut: AppShortcut.filterOverviewLanes.symbol))
                }
            }
        }
        .alert(
            Localized.Overview.abandonTitle(title: viewModel.pendingAbandon?.title ?? ""),
            isPresented: isPresented($viewModel.pendingAbandon),
            presenting: viewModel.pendingAbandon
        ) { request in
            Button(Localized.Menu.abandon, role: .destructive) { viewModel.abandon(request) }
            Button(Localized.Common.cancel, role: .cancel) {}
        } message: { request in
            Text(request.message)
        }
        .alert(
            Localized.Repo.deleteWorkspaceTitle(name: viewModel.pendingWorkspaceDelete?.name ?? ""),
            isPresented: isPresented($viewModel.pendingWorkspaceDelete),
            presenting: viewModel.pendingWorkspaceDelete
        ) { workspace in
            Button(Localized.Repo.delete, role: .destructive) { removeWorkspace(workspace, deleteFromDisk: true) }
            Button(Localized.Common.cancel, role: .cancel) {}
        } message: { workspace in
            Text(Localized.Repo.deleteWorkspaceMessage(path: workspace.path))
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
        FilterField(
            text: $viewModel.filter,
            placeholder: Localized.Overview.filterLanes,
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
                emptyState(Localized.Overview.noMutableChangesEveryChangeIs)
            } else {
                GeometryReader { proxy in
                    ScrollView([.horizontal, .vertical]) {
                        OverviewCanvas(
                            lanes: snapshot.overview.lanes,
                            laneIds: viewModel.laneIds,
                            placement: viewModel.placement,
                            trunkName: viewModel.trunkName,
                            selectedLaneId: $viewModel.selectedLaneId,
                            selectedChangeId: $viewModel.selectedChangeId,
                            isLanePanelShown: $viewModel.isLanePanelShown,
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
                        .onTapGesture {
                            viewModel.selectedChangeId = nil
                            viewModel.isLanePanelShown = false
                        }
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
                .onKeyPress(characters: CharacterSet(charactersIn: "hjkl")) { press in
                    guard press.modifiers.isEmpty else { return .ignored }
                    switch press.characters {
                        case "h": viewModel.selectNeighbor(-1)
                        case "l": viewModel.selectNeighbor(1)
                        case "j": viewModel.selectChangeNeighbor(1)
                        default: viewModel.selectChangeNeighbor(-1)
                    }
                    return .handled
                }
                .onKeyPress(.escape) {
                    if viewModel.selectedChangeId != nil {
                        viewModel.selectedChangeId = nil
                    } else if viewModel.isLanePanelShown {
                        viewModel.isLanePanelShown = false
                    } else {
                        return .ignored
                    }
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
            LoadingStatus(label: Localized.Overview.loadingOverviewEllipsis)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
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
            stat(lanes.count, Localized.Overview.statLanes(count: lanes.count))
            stat(changeCount, Localized.Overview.statChanges(count: changeCount))
            stat(Int(overview?.workspaceCount ?? 0), Localized.Overview.statWorkspaces(count: Int(overview?.workspaceCount ?? 0)))
            stat(behind, Localized.Overview.statBehindTrunk, tint: behind > 0 ? .orange : nil)
            stat(attention, Localized.Overview.statAttention(count: attention), tint: attention > 0 ? .orange : nil)
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

    private func stat(_ count: Int, _ label: String, tint: Color? = nil) -> some View {
        HStack(spacing: 4) {
            Text("\(count)")
                .jayjayFont(11, weight: .semibold)
            Text(label)
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
            headChangeId: lane.head.changeId.id,
            selecting: (change ?? lane.head).commitId.id
        )
    }
}
