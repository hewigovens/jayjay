import JayJayCore
import SwiftUI

struct RepoWindow: View {
    let repoPath: String
    @State private var viewModel: RepoViewModel?
    @State private var initError: String?
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Group {
            if let model = viewModel {
                RepoContentView(viewModel: model)
            } else if let err = initError {
                RepoInitErrorView(repoPath: repoPath, error: err, onInitialize: initJJRepo)
            } else {
                ProgressView("Loading repository...")
            }
        }
        .task { await openRepo() }
        .navigationTitle(URL(fileURLWithPath: repoPath).lastPathComponent)
        .background(WindowRepresentedURL(path: repoPath))
    }

    private func openRepo() async {
        let path = repoPath
        let includeSubmodules = settings.enableGitSubmoduleSupport
        // Off the main thread so the app stays responsive while loading large checkouts.
        let result = await Task.detached {
            Result {
                let repo = try JayJayRepo.open(path: path)
                return (
                    repo: repo,
                    workingCopyIsLarge: repo.workingCopyIsLarge(),
                    configWarning: repo.checkUserConfig()
                )
            }
        }.value
        switch result {
            case let .success(opened):
                let model = RepoViewModel(
                    path: path,
                    repo: opened.repo,
                    workingCopyIsLarge: opened.workingCopyIsLarge,
                    configWarning: opened.configWarning,
                    includeSubmoduleStatuses: includeSubmodules
                )
                viewModel = model
                // Huge checkouts skip the snapshot on open (it's the slow part); small repos refresh eagerly.
                model.refresh(selecting: "@", snapshotWorkingCopy: !model.workingCopyIsLarge)
            case let .failure(error):
                initError = error.friendlyDescription
        }
    }

    private func initJJRepo() {
        let status = checkJjEnvironment()
        guard status.isInstalled, !status.path.isEmpty else {
            initError = "jj is not installed"
            return
        }
        let proc = Process()
        proc.executableURL = URL(fileURLWithPath: status.path)
        proc.arguments = ["git", "init"]
        proc.currentDirectoryURL = URL(fileURLWithPath: repoPath)
        try? proc.run()
        proc.waitUntilExit()
        if proc.terminationStatus == 0 {
            initError = nil
            Task { await openRepo() }
        } else {
            initError = "Failed to initialize repository"
        }
    }
}

private struct RepoInitErrorView: View {
    let repoPath: String
    let error: String
    let onInitialize: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 40))
                .foregroundStyle(.orange)
            Text("Failed to open repository")
                .jayjayFont(16, weight: .semibold)
            Text(error)
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 360)
            if !FileManager.default.fileExists(atPath: "\(repoPath)/.jj") {
                Button("Initialize with jj git init", action: onInitialize)
                    .buttonStyle(.borderedProminent)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

/// Which pane keyboard navigation keys (j/k/arrows/Ctrl-n/Ctrl-p) target.
enum ActivePane {
    case dag, fileColumn
}

struct DAGRevealRequest: Equatable, Identifiable {
    let id = UUID()
    let changeId: String
}

struct RepoContentView: View {
    @Bindable var viewModel: RepoViewModel
    @State var revsetDraft = ""
    @State var showRevsetFilter = false
    @State var sidebarWidth: CGFloat = 360
    @State var bookmarkCreateName = ""
    @State var modal: RepoModalState?
    @State var workspaceName = ""
    @State var activePane: ActivePane = .dag
    @State var hasResetInitialFocus = false
    @State var dagRevealRequest: DAGRevealRequest?
    // @State for one stable panel per window: a plain `let` is re-evaluated on every
    // re-init (font/appearance changes), orphaning the visible panel and spawning a second.
    @State var commandPanel = CommandPalettePanel()
    @State var toast: RepoToastState?
    @State var toastDismissTask: Task<Void, Never>?
    @State var menuCoordinator = RepoMenuHandler()
    @Environment(AppSettings.self) var settings
    @Environment(RepoWindowManager.self) var windowManager
    @Environment(\.openSettings) var openSettings
    @Environment(\.colorScheme) var colorScheme

    var body: some View {
        contentLayout
            .frame(minWidth: 800, minHeight: 500)
            .onAppear {
                revsetDraft = viewModel.revset
                sidebarWidth = settings.sidebarWidth
                menuCoordinator.onAction = { action in
                    switch action {
                        case .commandPalette: showCommandPalette()
                        case .undo: showUndo()
                        case .bookmarkManager: modal = .bookmarkManager
                        case .newWorkspace: modal = .workspaceCreate
                    }
                }
                ActiveRepoTracker.shared.register(
                    repoPath: viewModel.repoPath, settings: settings, handler: menuCoordinator
                )
                // Defeat AppKit auto-focus on CommitBox so j/k nav works on cold launch.
                if !hasResetInitialFocus {
                    hasResetInitialFocus = true
                    Task { @MainActor in
                        try? await Task.sleep(for: .milliseconds(50))
                        NSApp.keyWindow?.makeFirstResponder(nil)
                    }
                }
            }
            .onChange(of: viewModel.revset) {
                revsetDraft = viewModel.revset
            }
            .onChange(of: viewModel.graphEntries.count) {
                // Auto-widen sidebar if graph has many lanes and user hasn't manually resized
                if settings.sidebarWidth <= 300 {
                    let layout = DAGLayout(entries: viewModel.graphEntries)
                    let lanes = layout.maxLanes()
                    let graphWidth = CGFloat(lanes) * laneWidth + 8
                    let minNeeded = min(160, graphWidth) + 250 // graph + text
                    if minNeeded > sidebarWidth {
                        sidebarWidth = min(500, minNeeded)
                    }
                }
            }
            .toolbar { toolbarContent }
            .overlay { presentationOverlay }
            .animation(.easeOut(duration: 0.3), value: toast?.id)
            .alert(alertTitle, isPresented: isAlertPresented, presenting: alertState) { alert in
                alertActions(for: alert)
            } message: { alert in
                Text(alertMessage(for: alert))
            }
            .onChange(of: viewModel.info) { _, msg in
                guard let msg, !msg.isEmpty else { return }
                showToast(msg)
                viewModel.info = nil
            }
            .sheet(item: $modal) { modal in
                modalView(for: modal)
            }
            .onChange(of: viewModel.successActionSignal) {
                handleSuccessActionSignalChange()
            }
            .onChange(of: viewModel.submoduleAttentionItems.count) {
                handleSubmoduleAttentionChange()
            }
    }

    private var contentLayout: some View {
        VStack(spacing: 0) {
            GeometryReader { geo in
                HStack(spacing: 0) {
                    sidebar.frame(width: sidebarWidth)
                    SidebarDivider(position: $sidebarWidth, range: 240 ... max(240, min(600, geo.size.width - 400)))
                    DetailView(
                        repoPath: viewModel.repoPath, repo: viewModel.repo,
                        detail: viewModel.selectedChange,
                        actions: viewModel,
                        onDescribe: { rev, msg in viewModel.describe(rev: rev, message: msg) },
                        reviewStore: viewModel.reviewStore,
                        diffStore: viewModel.diffStore,
                        compareFromId: viewModel.compareFromId,
                        compareDisplay: viewModel.compareDisplay,
                        onClearCompare: { viewModel.clearCompare() },
                        onReverseCompare: { viewModel.reverseCompare() },
                        onRevealChangeInDag: revealChangeInDAG,
                        activePane: $activePane,
                        evologEntries: viewModel.evologEntries,
                        evologRev: viewModel.evologRev,
                        onDismissEvolog: { viewModel.dismissEvolog() }
                    )
                    .frame(maxWidth: .infinity)
                }
            }
            Divider()
            statusBar
        }
    }

    private func revealChangeInDAG(_ changeId: String) {
        activePane = .dag
        dagRevealRequest = DAGRevealRequest(changeId: changeId)
        viewModel.select(changeId: changeId)
    }
}

@MainActor
final class RepoMenuHandler: RepositoryMenuHandler {
    var onAction: ((MenuAction) -> Void)?

    enum MenuAction {
        case commandPalette, undo, bookmarkManager, newWorkspace
    }

    func showCommandPalette() {
        onAction?(.commandPalette)
    }

    func showUndo() {
        onAction?(.undo)
    }

    func showBookmarkManager() {
        onAction?(.bookmarkManager)
    }

    func showNewWorkspace() {
        onAction?(.newWorkspace)
    }
}
