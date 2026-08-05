import JayJayCore
import SwiftUI

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
    @State var diffCommands = DiffCommands()
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
            .environment(diffCommands)
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
