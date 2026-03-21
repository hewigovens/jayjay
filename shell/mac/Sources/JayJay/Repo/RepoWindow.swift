import SwiftUI
import JayJayBindings

struct RepoWindow: View {
    let repoPath: String
    @State private var viewModel: RepoViewModel?
    @State private var initError: String?

    var body: some View {
        Group {
            if let model = viewModel {
                RepoContentView(viewModel: model)
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
            do {
                let model = try RepoViewModel(path: repoPath); viewModel = model; model.refresh()
            } catch {
                initError = error.localizedDescription
            }
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
    @State private var confirmAbandonRev: String?
    @State private var showUndoSheet = false
    @State private var toastMessage: String?
    @Environment(AppSettings.self) private var settings
    @Environment(RepoWindowManager.self) private var windowManager
    @Environment(\.openSettings) private var openSettings
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        VStack(spacing: 0) {
            GeometryReader { geo in
                HStack(spacing: 0) {
                    sidebar.frame(width: sidebarWidth)
                    SidebarDivider(position: $sidebarWidth, range: 240...max(240, min(600, geo.size.width - 400)))
                    DetailView(
                        repoPath: viewModel.repoPath, repo: viewModel.repo,
                        detail: viewModel.selectedChange,
                        actions: viewModel,
                        onDescribe: { rev, msg in viewModel.describe(rev: rev, message: msg) }
                    )
                    .frame(maxWidth: .infinity)
                }
            }
            Divider()
            statusBar
        }
        .onAppear {
            revsetDraft = viewModel.revset; sidebarWidth = settings.sidebarWidth
        }
        .focusedSceneValue(\.jayjayGitFetch) { viewModel.gitFetch() }
        .focusedSceneValue(\.jayjayGitPush) { viewModel.gitPush() }
        .focusedSceneValue(\.jayjaySettings, settings)
        .toolbar { toolbarContent }
        .overlay { if viewModel.isLoading { ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity).background(.ultraThinMaterial) } }
        .overlay {
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
                    .transition(.scale(scale: 0.9).combined(with: .opacity))
            }
        }
        .animation(.easeOut(duration: 0.2), value: toastMessage)
        .alert("Error", isPresented: .init(get: { viewModel.error != nil }, set: { if !$0 { viewModel.error = nil } })) {
            Button("OK") { viewModel.error = nil }
        } message: { Text(viewModel.error ?? "") }
        .onChange(of: viewModel.info) { _, msg in
            guard let msg, !msg.isEmpty else { return }
            showToast(msg)
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
        .sheet(isPresented: .init(get: { confirmAbandonRev != nil }, set: { if !$0 { confirmAbandonRev = nil } })) {
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
        .sheet(isPresented: $showUndoSheet) {
            UndoView(
                entries: viewModel.opLogEntries,
                onRestore: { opId in viewModel.opRestore(opId: opId) },
                onDismiss: { showUndoSheet = false }
            )
        }
        .focusedSceneValue(\.jayjayShowUndo) { showUndo() }
    }

    private func showUndo() {
        viewModel.opLog()
        showUndoSheet = true
    }

    private func requestAbandon(_ rev: String) {
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

    private func showToast(_ message: String) {
        toastMessage = message
        Task {
            try? await Task.sleep(for: .seconds(2))
            toastMessage = nil
        }
    }

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItemGroup(placement: .navigation) {
            BookmarkPicker(bookmarks: viewModel.bookmarks,
                           actions: viewModel,
                           onSelect: { revsetDraft = $0; applyRevset() })
            Button { showRevsetFilter.toggle() } label: {
                Label("Filter", systemImage: "line.3.horizontal.decrease.circle")
            }.help("Filter by revset")
            Button { viewModel.refresh() } label: {
                Label("Refresh", systemImage: "arrow.triangle.2.circlepath")
            }.keyboardShortcut("r").help("Refresh (⌘R)")
        }

        ToolbarItemGroup(placement: .primaryAction) {
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
                    actions: viewModel,
                    onAbandon: { requestAbandon($0) },
                    onCreateBookmark: { rev in bookmarkCreateRev = rev; bookmarkCreateName = "" })
            Divider()
            CommitBox(description: viewModel.workingCopyDescription,
                      onCommit: { viewModel.commit(message: $0) },
                      onGenerateMessage: { await viewModel.generateCommitMessage() },
                      aiProvider: viewModel.aiProvider)
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
