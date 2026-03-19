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
    }
}

struct RepoContentView: View {
    @Bindable var viewModel: RepoViewModel
    @State private var revsetDraft = ""
    @State private var showRevsetFilter = false
    @State private var sidebarWidth: CGFloat = 300
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
                        onSplit: { rev, paths in viewModel.split(rev: rev, paths: paths) }
                    )
                    .frame(maxWidth: .infinity)
                }
            }
            Divider()
            statusBar
        }
        .onAppear { revsetDraft = viewModel.revset; sidebarWidth = settings.sidebarWidth }
        .toolbar { toolbarContent }
        .overlay { if viewModel.isLoading { ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity).background(.ultraThinMaterial) } }
        .alert("Error", isPresented: .init(get: { viewModel.error != nil }, set: { if !$0 { viewModel.error = nil } })) {
            Button("OK") { viewModel.error = nil }
        } message: { Text(viewModel.error ?? "") }
    }

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItem(placement: .navigation) {
            BookmarkPicker(bookmarks: viewModel.bookmarks,
                           onSelect: { revsetDraft = $0; applyRevset() },
                           onCreate: { viewModel.createBookmark(name: $0) },
                           onDelete: { viewModel.deleteBookmark(name: $0) })
        }
        ToolbarItemGroup {
            Button { showRevsetFilter.toggle() } label: { Label("Filter", systemImage: "line.3.horizontal.decrease.circle") }.help("Filter by revset")
            Button { viewModel.refresh() } label: { Label("Refresh", systemImage: "arrow.clockwise") }.keyboardShortcut("r").help("Refresh (⌘R)")
            Spacer()
            Button { viewModel.gitFetch() } label: { Label("Fetch", systemImage: "arrow.down.circle") }.keyboardShortcut("f", modifiers: [.command, .shift]).help("Git Fetch (⌘⇧F)")
            Button { viewModel.gitPush() } label: { Label("Push", systemImage: "arrow.up.circle") }.keyboardShortcut("p", modifiers: [.command, .shift]).help("Git Push (⌘⇧P)")
            Spacer()
            Button { if let id = viewModel.selectedChangeId { viewModel.newChange(parent: id) } } label: { Label("New", systemImage: "plus") }.keyboardShortcut("n").disabled(viewModel.selectedChangeId == nil).help("New change (⌘N)")
            Button { if let id = viewModel.selectedChangeId { viewModel.squash(rev: id) } } label: { Label("Squash", systemImage: "square.and.arrow.down.on.square") }.keyboardShortcut("s", modifiers: [.command, .shift]).disabled(viewModel.selectedChangeId == nil).help("Squash into parent (⌘⇧S)")
            Button { if let id = viewModel.selectedChangeId { viewModel.abandon(rev: id) } } label: { Label("Abandon", systemImage: "trash") }.keyboardShortcut(.delete).disabled(viewModel.selectedChangeId == nil).help("Abandon change (⌘⌫)")
            Button { openSettings() } label: { Label("Settings", systemImage: "gearshape") }.help("Settings")
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
                    onAbandon: { viewModel.abandon(rev: $0) })
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
        guard !t.isEmpty else { return }
        revsetDraft = t; viewModel.applyRevset(t)
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
