import JayJayCore
import SwiftUI

struct RepoContentView: View {
    @Bindable var viewModel: RepoViewModel
    let isSwitchingWorkspace: Bool
    let onSwitchWorkspace: (String) -> Void
    @State var sidebarWidth: CGFloat = 360
    /// The sidebar's width on screen, which a narrow window holds below `sidebarWidth`.
    @State var sidebarShownWidth: CGFloat = 360
    @State var toolbarRoom = ToolbarRoom()
    @State var revsetEditRequest = 0
    @State var refCreateName = ""
    @State var modal: RepoModalState?
    @State var operationLogUndoing: String?
    @State var operationLogSelection: String?
    @State var sidebarReveal = PaneReveal(shown: true)
    @State var operationLog = PaneReveal(shown: UserDefaults.standard.bool(forKey: AppSettings.showsOperationLogKey))
    @State var detailInteractionActive = false
    @State var workspaceName = ""
    @State var workspaceNameError: String?
    @State var workspaceCreating = false
    @State var keyboardFocus = KeyboardFocus()
    @State var hasResetInitialFocus = false
    @State var diffCommands = DiffCommands()
    // @State for one stable panel per window: a plain `let` is re-evaluated on every
    // re-init (font/appearance changes), orphaning the visible panel and spawning a second.
    @State var commandPanel = CommandPalettePanel()
    @State var toast: RepoToastState?
    @State var toastDismissTask: Task<Void, Never>?
    @State var menuCoordinator = RepoMenuHandler()
    @Environment(AppSettings.self) var settings
    @AppStorage(AppSettings.showsStatusBarKey) var showsStatusBar = true
    @AppStorage(AppSettings.showsOperationLogKey) var showsOperationLog = false
    @AppStorage(AppSettings.operationLogWidthKey) var storedOperationLogWidth = Double(PaneLayout.operationLogDefault)
    @Environment(RepoWindowManager.self) var windowManager
    @Environment(\.openSettings) var openSettings
    @Environment(\.colorScheme) var colorScheme
    @Environment(\.accessibilityReduceMotion) var reduceMotion

    var body: some View {
        contentLayout
            .frame(minWidth: 800, minHeight: 500)
            .environment(diffCommands)
            .onAppear {
                sidebarWidth = settings.sidebarWidth
                menuCoordinator.onAction = { action in
                    switch action {
                        case let .shortcut(shortcut): performShortcut(shortcut)
                        case .commandPalette: showCommandPalette()
                        case .bookmarkManager: modal = .bookmarkManager
                        case .revsetFilter: revsetEditRequest += 1
                        case .overview: windowManager.openOverview(for: viewModel.repoPath)
                        case .newWorkspace: modal = .workspaceCreate
                        case .pullRequestImport: modal = .pullRequestImport
                        case .restoreOperation: restorableOperation.map(restoreOperation)
                    }
                }
                menuCoordinator.viewModel = viewModel
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
            .onChange(of: availableMenuShortcuts, initial: true) { _, shortcuts in
                menuCoordinator.availableShortcuts = shortcuts
            }
            .onChange(of: viewModel.selectedReviewMode, initial: true) { _, mode in
                menuCoordinator.reviewMode = mode
            }
            .onChange(of: viewModel.hasWebRemote, initial: true) { _, hasWebRemote in
                menuCoordinator.canViewRemoteRepository = hasWebRemote
            }
            .onChange(of: viewModel.revset) {
                showSidebar()
            }
            .onChange(of: viewModel.dagRevealRequest?.id) {
                showSidebar()
                keyboardFocus.activePane = .dag
            }
            .onChange(of: settings.sidebarHidden, initial: true) { wasHidden, hidden in
                handleSidebarVisibilityChange(hidden: hidden, animated: wasHidden != hidden)
            }
            .onChange(of: viewModel.workspaceVanished) { _, vanished in
                guard vanished else { return }
                let repoPath = viewModel.repoPath
                Task { @MainActor in
                    await windowManager.withWorkspaceRemoval(at: repoPath) {}
                }
            }
            .toolbar { toolbarContent }
            // The system otherwise fills the toolbar with its own background once a pane has toggled.
            .toolbarBackgroundVisibility(.hidden, for: .windowToolbar)
            .background(TitleBarDoubleClick().frame(width: 0, height: 0).allowsHitTesting(false))
            .environment(keyboardFocus)
            .background(
                KeyDownMonitor(
                    yieldsToText: { _ in keyboardFocus.control?.ownsKeys != true },
                    swallowsUnhandledKeys: keyboardFocus.control.map { !$0.ownsKeys } ?? false,
                    onKeyDown: { event in keyboardFocus.handleKey(event) }
                )
                .frame(width: 0, height: 0)
                .allowsHitTesting(false)
            )
            .overlay { presentationOverlay }
            .animation(.easeOut(duration: 0.3), value: toast?.id)
            .alert(Localized.Repo.error, isPresented: isErrorPresented, presenting: viewModel.error) { _ in
                Button(Localized.Repo.ok) { viewModel.error = nil }
            } message: { message in
                Text(message)
            }
            .onChange(of: viewModel.info) { _, msg in
                guard let msg, !msg.isEmpty else { return }
                showToast(msg)
                viewModel.info = nil
            }
            .onChange(of: viewModel.actionNotice) { _, notice in
                guard let notice else { return }
                showActionNotice(notice)
                viewModel.actionNotice = nil
            }
            .onChange(of: viewModel.operationLogRequest) { _, operationId in
                guard let operationId else { return }
                viewModel.operationLogRequest = nil
                showOperationLog(undoing: operationId)
            }
            .sheet(item: $modal) { modal in
                modalView(for: modal)
            }
            .onReceive(NotificationCenter.default.publisher(for: .jjIdentitySaved)) { _ in
                viewModel.refresh(isAutoTriggered: true)
            }
            .onChange(of: viewModel.successActionSignal) {
                handleSuccessActionSignalChange()
            }
            .onChange(of: backgroundRefreshSuspended, initial: true) { _, suspended in
                viewModel.setBackgroundRefreshSuspended(suspended)
            }
            .onDisappear {
                viewModel.setBackgroundRefreshSuspended(false)
            }
    }

    private var availableMenuShortcuts: Set<AppShortcut> {
        availableShortcuts(among: RepositoryCommands.menuShortcuts)
    }

    private var contentLayout: some View {
        VStack(spacing: 0) {
            HStack(spacing: 0) {
                VStack(spacing: 0) {
                    SSHReconnectBar(viewModel: viewModel)
                    BisectBar(viewModel: viewModel)
                    ResizableSplit(
                        width: $sidebarWidth,
                        isLeadingHidden: settings.sidebarHidden,
                        range: PaneLayout.sidebarRange(windowWidth:),
                        onEnded: { settings.sidebarWidth = $0 },
                        dividerIdentifier: AID.Sidebar.divider,
                        leading: {
                            sidebar
                                .onGeometryChange(for: CGFloat.self) { $0.size.width } action: { sidebarShownWidth = $0 }
                        },
                        trailing: {
                            DetailView(
                                repoPath: viewModel.repoPath, repo: viewModel.repo,
                                detail: viewModel.selectedChange,
                                actions: viewModel,
                                onEditDescription: { rev, description in modal = .editDescription(rev: rev, description: description) },
                                reviewStore: viewModel.reviewStore,
                                diffStore: viewModel.diffStore,
                                comparison: viewModel.comparison,
                                onClearCompare: { viewModel.clearCompare() },
                                onReverseCompare: viewModel.canReverseCompare
                                    ? { viewModel.reverseCompare() } : nil,
                                onRevealChangeInDag: { viewModel.revealInGraph($0) },
                                onResolveDivergent: { modal = .converge(rev: $0) },
                                divergentVersionCount: viewModel.selectedChange.map { viewModel.divergentVersionCount(of: $0.info.changeId.id) } ?? 0,
                                activePane: Bindable(keyboardFocus).activePane,
                                evolog: viewModel.evolog,
                                evologTarget: viewModel.evolog.flatMap { viewModel.change(for: $0.rev) },
                                isRestoringEvologVersion: viewModel.isRestoringEvologVersion,
                                onRestoreEvologVersion: { viewModel.restoreEvologVersion($0) },
                                onDismissEvolog: { viewModel.dismissEvolog() },
                                conflictedBookmarkNames: viewModel.conflictedBookmarkNames,
                                selectionWithoutDiffCount: viewModel.comparison == nil
                                    ? viewModel.selectedChangeIds.count : 0,
                                onInteractionStateChanged: { detailInteractionActive = $0 }
                            )
                            .background(paneBackground)
                        }
                    )
                    .disabled(!viewModel.sshConnected)
                }
                TrailingPaneColumn(
                    reveal: operationLog,
                    width: Binding { toolbarRoom.operationLogWidth } set: { toolbarRoom.preferredOperationLogWidth = $0 },
                    range: { toolbarRoom.operationLogRange },
                    onResizeEnded: { _ in storedOperationLogWidth = toolbarRoom.preferredOperationLogWidth },
                    content: { operationLogPanel }
                )
                .disabled(!viewModel.sshConnected)
            }
            if showsStatusBar {
                Divider()
                statusBar
            }
        }
        .background(paneBackground)
        .onGeometryChange(for: CGFloat.self) { $0.size.width } action: { toolbarRoom.toolbarWidth = $0 }
        .onChange(of: restorableOperation, initial: true) { _, operation in
            menuCoordinator.canRestoreOperation = operation != nil
        }
        .onChange(of: showsOperationLog) { _, shown in
            operationLog.animate(toShown: shown, in: windowManager.repoWindow(at: viewModel.repoPath), animated: !reduceMotion)
        }
    }

    private var paneBackground: Color {
        settings.navigationBackground(colorScheme)
    }

    /// Alerts deliberately don't suspend: pausing on an error would make dismissal re-run the failing refresh.
    private var backgroundRefreshSuspended: Bool {
        modal != nil || detailInteractionActive
    }
}
