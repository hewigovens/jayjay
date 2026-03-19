import SwiftUI
import JayJayBindings

struct RepoWindow: View {
    let repoPath: String
    @State private var viewModel: RepoViewModel?
    @State private var initError: String?

    var body: some View {
        Group {
            if let vm = viewModel {
                RepoContentView(viewModel: vm)
            } else if let err = initError {
                VStack(spacing: 12) {
                    Image(systemName: "exclamationmark.triangle").font(.largeTitle).foregroundStyle(.red)
                    Text("Failed to open repository").jayjayFont(18, weight: .semibold)
                    Text(err).foregroundStyle(.secondary).textSelection(.enabled)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ProgressView("Loading repository...")
            }
        }
        .task {
            do { let vm = try RepoViewModel(path: repoPath); viewModel = vm; vm.refresh() }
            catch { initError = error.localizedDescription }
        }
        .navigationTitle(URL(fileURLWithPath: repoPath).lastPathComponent)
        .focusedSceneValue(\.jayjayRepoPath, repoPath)
    }
}

struct RepoContentView: View {
    @Bindable var viewModel: RepoViewModel
    @State private var revsetDraft = ""
    @State private var showRevsetFilter = false
    @State private var sidebarWidth: CGFloat = 300
    @State private var bookmarkCreateRev: String?
    @State private var bookmarkCreateName = ""
    @Environment(AppSettings.self) private var settings
    @Environment(RepoWindowManager.self) private var windowManager
    @Environment(\.openSettings) private var openSettings

    var body: some View {
        VStack(spacing: 0) {
            GeometryReader { geo in
                HStack(spacing: 0) {
                    sidebar.frame(width: sidebarWidth)
                    SidebarDivider(position: $sidebarWidth, range: 240...min(600, geo.size.width - 400))
                    DetailView(
                        repoPath: viewModel.repoPath, repo: viewModel.repo,
                        detail: viewModel.selectedChange,
                        onDescribe: { rev, msg in viewModel.describe(rev: rev, message: msg) },
                        onRestoreFiles: { rev, paths in viewModel.restoreFiles(rev: rev, paths: paths) },
                        onIgnoreAndUntrack: { paths in viewModel.ignoreAndUntrack(paths: paths) },
                        onSplit: { rev, paths, msg in viewModel.split(rev: rev, paths: paths, message: msg) }
                    )
                    .frame(maxWidth: .infinity)
                }
            }
            Divider()
            statusBar
        }
        .onAppear { revsetDraft = viewModel.revset; sidebarWidth = settings.sidebarWidth }
        .focusedSceneValue(\.jayjayGitFetch) { viewModel.gitFetch() }
        .focusedSceneValue(\.jayjayGitPush) { viewModel.gitPush() }
        .toolbar { toolbarContent }
        .overlay { if viewModel.isLoading { ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity).background(.ultraThinMaterial) } }
        .alert("Error", isPresented: .init(get: { viewModel.error != nil }, set: { if !$0 { viewModel.error = nil } })) {
            Button("OK") { viewModel.error = nil }
        } message: { Text(viewModel.error ?? "") }
        .onChange(of: viewModel.info) { _, msg in
            guard let msg, !msg.isEmpty else { return }
            showNotification(msg)
            viewModel.info = nil
        }
        .sheet(isPresented: .init(get: { bookmarkCreateRev != nil }, set: { if !$0 { bookmarkCreateRev = nil } })) {
            VStack(alignment: .leading, spacing: 10) {
                Text("Create Bookmark").jayjayFont(14, weight: .semibold)
                Text("On change: \(String(bookmarkCreateRev?.prefix(12) ?? ""))").jayjayFont(11, design: .monospaced).foregroundStyle(.secondary)
                TextField("Bookmark name", text: $bookmarkCreateName)
                    .textFieldStyle(.roundedBorder).jayjayFont(13, design: .monospaced).frame(width: 260)
                    .onSubmit { submitBookmarkCreate() }
                HStack {
                    Spacer()
                    Button("Cancel") { bookmarkCreateRev = nil }.keyboardShortcut(.cancelAction)
                    Button("Create") { submitBookmarkCreate() }.keyboardShortcut(.defaultAction)
                        .disabled(bookmarkCreateName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }
            .padding(20)
        }
    }

    private func submitBookmarkCreate() {
        let name = bookmarkCreateName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty, let rev = bookmarkCreateRev else { return }
        viewModel.createBookmark(name: name, rev: rev)
        bookmarkCreateRev = nil
    }

    private func showNotification(_ message: String) {
        let notification = NSUserNotification()
        notification.title = "JayJay"
        notification.informativeText = message
        notification.soundName = nil
        NSUserNotificationCenter.default.deliver(notification)
    }

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        // Left: Bookmark + Filter + Refresh
        ToolbarItemGroup(placement: .navigation) {
            BookmarkPicker(bookmarks: viewModel.bookmarks,
                           onSelect: { revsetDraft = $0; applyRevset() },
                           onCreate: { viewModel.createBookmark(name: $0) },
                           onDelete: { viewModel.deleteBookmark(name: $0) },
                           onPush: { viewModel.gitPush(bookmark: $0) },
                           onFetch: { viewModel.gitFetch() })
            Button { showRevsetFilter.toggle() } label: {
                Label("Filter", systemImage: "line.3.horizontal.decrease.circle")
            }.help("Filter by revset")
            Button { viewModel.refresh() } label: {
                Label("Refresh", systemImage: "arrow.clockwise")
            }.keyboardShortcut("r").help("Refresh (⌘R)")
        }

        // Right: New + Squash + Abandon + Settings
        ToolbarItemGroup(placement: .primaryAction) {
            Button { if let id = viewModel.selectedChangeId { viewModel.newChange(parent: id) } } label: {
                Label("New", systemImage: "plus")
            }.keyboardShortcut("n").disabled(viewModel.selectedChangeId == nil).help("New change (⌘N)")
            Button { if let id = viewModel.selectedChangeId { viewModel.squash(rev: id) } } label: {
                Label("Squash", systemImage: "square.and.arrow.down.on.square")
            }.keyboardShortcut("s", modifiers: [.command, .shift]).disabled(viewModel.selectedChangeId == nil).help("Squash into parent (⌘⇧S)")
            Button { if let id = viewModel.selectedChangeId { viewModel.abandon(rev: id) } } label: {
                Label("Abandon", systemImage: "trash")
            }.keyboardShortcut(.delete).disabled(viewModel.selectedChangeId == nil).help("Abandon change (⌘⌫)")
            Button { openSettings() } label: {
                Label("Settings", systemImage: "gearshape")
            }.help("Settings")
        }
    }

    private var sidebar: some View {
        VStack(spacing: 0) {
            if showRevsetFilter {
                HStack(spacing: 6) {
                    TextField("Revset filter", text: $revsetDraft)
                        .textFieldStyle(.roundedBorder).jayjayFont(12, design: .monospaced).onSubmit { applyRevset() }
                    Button { applyRevset() } label: { Image(systemName: "arrow.right.circle.fill").foregroundStyle(.secondary) }
                        .buttonStyle(.plain).disabled(revsetDraft == viewModel.revset)
                }
                .padding(.horizontal, 12).padding(.vertical, 8)
                Divider()
            }
            DAGView(entries: viewModel.graphEntries, selectedId: viewModel.selectedChangeId,
                    onSelect: { viewModel.select(changeId: $0) },
                    onNew: { viewModel.newChange(parent: $0) },
                    onSquash: { viewModel.squash(rev: $0) },
                    onAbandon: { viewModel.abandon(rev: $0) },
                    onCreateBookmark: { rev in bookmarkCreateRev = rev; bookmarkCreateName = "" })
            Divider()
            CommitBox(description: viewModel.workingCopyDescription,
                      onCommit: { viewModel.commit(message: $0) },
                      onGenerateMessage: { await viewModel.generateCommitMessage() })
        }
    }

    private var statusBar: some View {
        HStack(spacing: 12) {
            Text(viewModel.repoPath).lineLimit(1).truncationMode(.middle)
            Spacer()
            Text("\(viewModel.changes.count) changes")
        }
        .jayjayFont(11).foregroundStyle(.secondary)
        .padding(.horizontal, 12).padding(.vertical, 5).background(.bar)
    }

    private func applyRevset() {
        let t = revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        if t.isEmpty {
            // Reset to default
            let defaultRevset = "@ | ancestors(@, 20) | @-+"
            revsetDraft = defaultRevset
            viewModel.applyRevset(defaultRevset)
        } else {
            revsetDraft = t
            viewModel.applyRevset(t)
        }
    }
}

private struct SidebarDivider: View {
    @Binding var position: CGFloat
    let range: ClosedRange<CGFloat>
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Rectangle()
            .fill(Color.primary.opacity(0.08))
            .frame(width: 1)
            .contentShape(Rectangle().inset(by: -3))
            .onHover { if $0 { NSCursor.resizeLeftRight.push() } else { NSCursor.pop() } }
            .gesture(DragGesture(minimumDistance: 1)
                .onChanged { position = min(max(position + $0.translation.width, range.lowerBound), range.upperBound) }
                .onEnded { _ in settings.sidebarWidth = position })
    }
}
