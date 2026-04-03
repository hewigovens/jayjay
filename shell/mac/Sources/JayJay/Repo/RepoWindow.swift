import JayJayCore
import SwiftUI

struct RepoWindow: View {
    let repoPath: String
    @State private var viewModel: RepoViewModel?
    @State private var initError: String?

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
        .task { openRepo() }
        .navigationTitle(URL(fileURLWithPath: repoPath).lastPathComponent)
    }

    private func openRepo() {
        do {
            let model = try RepoViewModel(path: repoPath)
            viewModel = model
            model.refresh(selecting: "@")
        } catch {
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
            openRepo()
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

struct RepoContentView: View {
    @Bindable var viewModel: RepoViewModel
    @State var revsetDraft = ""
    @State var showRevsetFilter = false
    @State var sidebarWidth: CGFloat = 360
    @State var bookmarkCreateRev: String?
    @State var bookmarkCreateName = ""
    @State var confirmAbandonRev: String?
    @State var showUndoSheet = false
    @State var showBookmarkManager = false
    @State var showWorkspaceCreate = false
    @State var workspaceName = ""
    @State var showSponsorPrompt = false
    let commandPanel = CommandPalettePanel()
    @State var toastMessage: String?
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
                        case .bookmarkManager: showBookmarkManager = true
                        case .newWorkspace: showWorkspaceCreate = true
                    }
                }
                ActiveRepoTracker.shared.register(
                    repoPath: viewModel.repoPath, settings: settings, handler: menuCoordinator
                )
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
            .overlay { loadingOverlay }
            .overlay { toastOverlay }
            .animation(.easeOut(duration: 0.3), value: toastMessage)
            .modifier(RepoAlertsModifier(viewModel: viewModel, openSettings: openSettings))
            .onChange(of: viewModel.info) { _, msg in
                guard let msg, !msg.isEmpty else { return }
                showToast(msg)
                viewModel.info = nil
            }
            .sheet(isPresented: bookmarkSheetPresented) { bookmarkCreateSheet }
            .sheet(isPresented: abandonSheetPresented) { abandonSheet }
            .sheet(isPresented: submoduleSheetPresented) { submoduleAttentionSheet }
            .sheet(isPresented: $showUndoSheet) {
                UndoView(
                    entries: viewModel.opLogEntries,
                    onRestore: { opId in viewModel.opRestore(opId: opId) },
                    onDismiss: { showUndoSheet = false }
                )
            }
            .sheet(isPresented: $showBookmarkManager) {
                BookmarkManagerView(
                    bookmarks: viewModel.bookmarks,
                    actions: viewModel,
                    repo: viewModel.repo,
                    onCleanUp: { viewModel.forgetStaleBookmarks() },
                    onFilter: { bookmarkName in
                        showBookmarkManager = false
                        revsetDraft = "ancestors(\(bookmarkName), 20) | trunk()"
                        applyRevset()
                    },
                    onDismiss: { showBookmarkManager = false }
                )
            }
            .sheet(isPresented: $showWorkspaceCreate) { workspaceCreateSheet }
            .modifier(SponsorPromptModifier(
                signal: viewModel.successActionSignal,
                settings: settings,
                isPresented: $showSponsorPrompt
            ))
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
                        onClearCompare: { viewModel.clearCompare() }
                    )
                    .frame(maxWidth: .infinity)
                }
            }
            Divider()
            statusBar
        }
    }

    private var loadingOverlay: some View {
        Group {
            if viewModel.isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(.ultraThinMaterial)
            }
        }
    }

    private var toastOverlay: some View {
        Group {
            if let toast = toastMessage {
                Text(toast)
                    .jayjayFont(13, weight: .medium)
                    .foregroundStyle(colorScheme == .dark ? .white : .black)
                    .padding(.horizontal, 24)
                    .padding(.vertical, 14)
                    .background(
                        RoundedRectangle(cornerRadius: 14, style: .continuous)
                            .fill(colorScheme == .dark ? Color.black.opacity(0.75) : Color.white.opacity(0.9))
                            .shadow(color: .black.opacity(0.2), radius: 12, y: 6)
                    )
                    .onTapGesture { toastDismissTask?.cancel()
                        toastMessage = nil
                    }
                    .transition(.scale(scale: 0.9).combined(with: .opacity))
            }
        }
    }

    private var bookmarkSheetPresented: Binding<Bool> {
        .init(
            get: { bookmarkCreateRev != nil },
            set: { if !$0 { bookmarkCreateRev = nil } }
        )
    }

    private var abandonSheetPresented: Binding<Bool> {
        .init(
            get: { confirmAbandonRev != nil },
            set: { if !$0 { confirmAbandonRev = nil } }
        )
    }

    private var submoduleSheetPresented: Binding<Bool> {
        .init(
            get: { !viewModel.submoduleAttentionItems.isEmpty },
            set: {
                if !$0 {
                    viewModel.submoduleAttentionItems = []
                    viewModel.pendingCommitMessage = nil
                }
            }
        )
    }

    private var bookmarkCreateSheet: some View {
        SheetContainer(
            title: "Create Bookmark",
            subtitle: "On change: \(String(bookmarkCreateRev?.prefix(12) ?? ""))",
            cancelLabel: "Cancel",
            confirmLabel: "Create",
            confirmDisabled: bookmarkCreateName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
            onCancel: { bookmarkCreateRev = nil },
            onConfirm: { submitBookmarkCreate() },
            content: {
                TextField("Bookmark name", text: $bookmarkCreateName)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(13, design: .monospaced)
                    .onSubmit { submitBookmarkCreate() }
            }
        )
    }

    private var abandonSheet: some View {
        VStack(spacing: 16) {
            Image(systemName: "trash.circle.fill")
                .font(.system(size: 36))
                .foregroundStyle(.red)
            Text("Abandon Change?")
                .jayjayFont(16, weight: .semibold)
            Text("This will remove the change and reparent its children.\nYou can undo this with jj op restore.")
                .jayjayFont(13)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            Toggle("Don't ask again", isOn: Binding(
                get: { settings.skipAbandonConfirmation },
                set: { settings.skipAbandonConfirmation = $0 }
            ))
            .jayjayFont(12)

            HStack(spacing: 12) {
                Button("Cancel") { confirmAbandonRev = nil }
                    .keyboardShortcut(.cancelAction)
                Button("Abandon") {
                    if let rev = confirmAbandonRev {
                        viewModel.abandon(rev: rev)
                        confirmAbandonRev = nil
                    }
                }
                .keyboardShortcut(.defaultAction)
                .buttonStyle(.borderedProminent)
                .tint(.red)
            }
        }
        .padding(24)
        .frame(width: 340)
    }

    private var workspaceCreateSheet: some View {
        SheetContainer(
            title: "New Workspace",
            subtitle: "Creates a new working copy in a sibling directory",
            cancelLabel: "Cancel",
            confirmLabel: "Create",
            confirmDisabled: workspaceName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
            onCancel: { showWorkspaceCreate = false },
            onConfirm: {
                let name = workspaceName.trimmingCharacters(in: .whitespacesAndNewlines)
                guard !name.isEmpty else { return }
                let parent = URL(fileURLWithPath: viewModel.repoPath).deletingLastPathComponent()
                let dest = parent.appendingPathComponent(name).path
                viewModel.workspaceAdd(dest: dest, name: name)
                showWorkspaceCreate = false
                workspaceName = ""
                windowManager.openRepo(dest)
            },
            content: {
                TextField("Workspace name", text: $workspaceName)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(13, design: .monospaced)
            }
        )
    }

    private var submoduleAttentionSheet: some View {
        SubmoduleAttentionSheet(
            repoPath: viewModel.repoPath,
            submoduleStatuses: viewModel.submoduleAttentionItems,
            onClose: {
                viewModel.submoduleAttentionItems = []
                viewModel.pendingCommitMessage = nil
            },
            onAutoCommit: { await viewModel.commitWithSafeSubmoduleUpdates() }
        )
    }

    func showUndo() {
        viewModel.opLog()
        showUndoSheet = true
    }

    func requestAbandon(_ rev: String) {
        if settings.skipAbandonConfirmation {
            viewModel.abandon(rev: rev)
        } else {
            confirmAbandonRev = rev
        }
    }

    private func submitBookmarkCreate() {
        let name = bookmarkCreateName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty, let rev = bookmarkCreateRev else { return }
        viewModel.createBookmark(name: name, rev: rev)
        bookmarkCreateRev = nil
    }

    func showToast(_ message: String) {
        toastDismissTask?.cancel()
        toastMessage = message
        toastDismissTask = Task {
            try? await Task.sleep(for: .seconds(2))
            guard !Task.isCancelled else { return }
            toastMessage = nil
        }
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

private struct RepoAlertsModifier: ViewModifier {
    @Bindable var viewModel: RepoViewModel
    let openSettings: OpenSettingsAction

    func body(content: Content) -> some View {
        content
            .alert("Error", isPresented: errorBinding) {
                Button("OK") { viewModel.error = nil }
            } message: { Text(viewModel.error ?? "") }
            .alert("jj Configuration Incomplete", isPresented: configWarningBinding) {
                Button("Open Settings") { openSettings() }
                Button("Dismiss", role: .cancel) { viewModel.configWarning = nil }
            } message: { Text(viewModel.configWarning ?? "") }
    }

    private var errorBinding: Binding<Bool> {
        .init(
            get: { viewModel.error != nil },
            set: { if !$0 { viewModel.error = nil } }
        )
    }

    private var configWarningBinding: Binding<Bool> {
        .init(
            get: { viewModel.configWarning != nil },
            set: { if !$0 { viewModel.configWarning = nil } }
        )
    }
}
