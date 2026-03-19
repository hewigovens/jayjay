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
                    Image(systemName: "exclamationmark.triangle")
                        .font(.largeTitle)
                        .foregroundStyle(.red)
                    Text("Failed to open repository")
                        .jayjayFont(18, weight: .semibold)
                    Text(err)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ProgressView("Loading repository...")
            }
        }
        .task {
            do {
                let vm = try RepoViewModel(path: repoPath)
                viewModel = vm
                vm.refresh()
            } catch {
                initError = error.localizedDescription
            }
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
                    sidebar
                        .frame(width: sidebarWidth)

                    SidebarDivider(position: $sidebarWidth, range: 240...min(600, geo.size.width - 400))

                    DetailView(
                        repoPath: viewModel.repoPath,
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
        .onAppear {
            revsetDraft = viewModel.revset
            sidebarWidth = settings.sidebarWidth
        }
        .toolbar {
            ToolbarItem(placement: .navigation) {
                BookmarkPicker(
                    bookmarks: viewModel.bookmarks,
                    onSelect: { name in
                        revsetDraft = name
                        applyRevset()
                    },
                    onCreate: { name in
                        viewModel.createBookmark(name: name)
                    },
                    onDelete: { name in
                        viewModel.deleteBookmark(name: name)
                    }
                )
            }
            ToolbarItemGroup {
                Button {
                    showRevsetFilter.toggle()
                } label: {
                    Label("Filter", systemImage: "line.3.horizontal.decrease.circle")
                }
                .help("Filter changes by revset")

                Button {
                    viewModel.refresh()
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .keyboardShortcut("r")
                .help("Refresh (⌘R)")

                Spacer()

                Button {
                    viewModel.gitFetch()
                } label: {
                    Label("Fetch", systemImage: "arrow.down.circle")
                }
                .keyboardShortcut("f", modifiers: [.command, .shift])
                .help("Git Fetch (⌘⇧F)")

                Button {
                    viewModel.gitPush()
                } label: {
                    Label("Push", systemImage: "arrow.up.circle")
                }
                .keyboardShortcut("p", modifiers: [.command, .shift])
                .help("Git Push (⌘⇧P)")

                Spacer()

                Button {
                    if let id = viewModel.selectedChangeId {
                        viewModel.newChange(parent: id)
                    }
                } label: {
                    Label("New", systemImage: "plus")
                }
                .keyboardShortcut("n")
                .disabled(viewModel.selectedChangeId == nil)
                .help("New change (⌘N)")

                Button {
                    if let id = viewModel.selectedChangeId {
                        viewModel.squash(rev: id)
                    }
                } label: {
                    Label("Squash", systemImage: "square.and.arrow.down.on.square")
                }
                .keyboardShortcut("s", modifiers: [.command, .shift])
                .disabled(viewModel.selectedChangeId == nil)
                .help("Squash into parent (⌘⇧S)")

                Button {
                    if let id = viewModel.selectedChangeId {
                        viewModel.abandon(rev: id)
                    }
                } label: {
                    Label("Abandon", systemImage: "trash")
                }
                .keyboardShortcut(.delete)
                .disabled(viewModel.selectedChangeId == nil)
                .help("Abandon change (⌘⌫)")

                Button {
                    showSettings()
                } label: {
                    Label("Settings", systemImage: "gearshape")
                }
                .help("Settings")
            }
        }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(.ultraThinMaterial)
            }
        }
        .alert("Error", isPresented: .init(
            get: { viewModel.error != nil },
            set: { if !$0 { viewModel.error = nil } }
        )) {
            Button("OK") { viewModel.error = nil }
        } message: {
            Text(viewModel.error ?? "")
        }
    }

    private var sidebar: some View {
        VStack(spacing: 0) {
            if showRevsetFilter {
                revsetBar
                Divider()
            }

            DAGView(
                entries: viewModel.graphEntries,
                selectedId: viewModel.selectedChangeId,
                onSelect: { viewModel.select(changeId: $0) },
                onNew: { viewModel.newChange(parent: $0) },
                onSquash: { viewModel.squash(rev: $0) },
                onAbandon: { viewModel.abandon(rev: $0) }
            )

            Divider()

            CommitBox(
                description: viewModel.workingCopyDescription,
                onDescribe: { viewModel.describeWorkingCopy(message: $0) },
                onCommit: { viewModel.commit(message: $0) },
                onGenerateMessage: { await viewModel.generateCommitMessage() }
            )
        }
    }

    private var revsetBar: some View {
        HStack(spacing: 6) {
            TextField("Revset filter", text: $revsetDraft)
                .textFieldStyle(.roundedBorder)
                .jayjayFont(12, design: .monospaced)
                .onSubmit { applyRevset() }
            Button {
                applyRevset()
            } label: {
                Image(systemName: "arrow.right.circle.fill")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .disabled(revsetDraft == viewModel.revset)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }

    private var statusBar: some View {
        HStack(spacing: 12) {
            Text(viewModel.repoPath)
                .lineLimit(1)
                .truncationMode(.middle)
            Spacer()
            Text("\(viewModel.changes.count) changes")
        }
        .jayjayFont(11)
        .foregroundStyle(.secondary)
        .padding(.horizontal, 12)
        .padding(.vertical, 5)
        .background(.bar)
    }

    private func applyRevset() {
        let trimmed = revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        revsetDraft = trimmed
        viewModel.applyRevset(trimmed)
    }

    private func showSettings() {
        openSettings()
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
            .cursor(.resizeLeftRight)
            .gesture(
                DragGesture(minimumDistance: 1)
                    .onChanged { value in
                        position = min(max(position + value.translation.width, range.lowerBound), range.upperBound)
                    }
                    .onEnded { _ in
                        settings.sidebarWidth = position
                    }
            )
    }
}

private extension View {
    func cursor(_ cursor: NSCursor) -> some View {
        onHover { inside in
            if inside { cursor.push() } else { NSCursor.pop() }
        }
    }
}

struct CommitBox: View {
    let description: String
    let onDescribe: (String) -> Void
    let onCommit: (String) -> Void
    let onGenerateMessage: () async -> String?

    @State private var draft = ""
    @State private var isGenerating = false

    private var trimmedDraft: String {
        draft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Description")
                    .jayjayFont(11, weight: .semibold)
                    .foregroundStyle(.secondary)
                Spacer()
                Button {
                    isGenerating = true
                    Task {
                        if let msg = await onGenerateMessage() {
                            draft = msg
                        }
                        isGenerating = false
                    }
                } label: {
                    if isGenerating {
                        ProgressView()
                            .controlSize(.mini)
                    } else {
                        Image(systemName: "sparkles")
                    }
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .help("Generate message with AI")
                .disabled(isGenerating)
            }

            TextEditor(text: $draft)
                .jayjayFont(13, design: .monospaced)
                .scrollContentBackground(.hidden)
                .padding(6)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(Color.primary.opacity(0.1), lineWidth: 1)
                )
                .frame(minHeight: 60, maxHeight: 120)

            HStack(spacing: 8) {
                Button {
                    if !trimmedDraft.isEmpty {
                        onDescribe(trimmedDraft)
                    }
                } label: {
                    Text("Describe")
                        .jayjayFont(12, weight: .medium)
                }
                .controlSize(.small)
                .disabled(trimmedDraft.isEmpty)
                .help("Update working copy description")

                Button {
                    if !trimmedDraft.isEmpty {
                        onCommit(trimmedDraft)
                    }
                } label: {
                    Text("Commit")
                        .jayjayFont(12, weight: .semibold)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(trimmedDraft.isEmpty)
                .help("Describe + start new change (jj commit)")
            }
            .frame(maxWidth: .infinity, alignment: .trailing)
        }
        .padding(12)
        .onAppear {
            draft = description
        }
        .onChange(of: description) { _, newValue in
            draft = newValue
        }
    }
}

struct BookmarkPicker: View {
    let bookmarks: [BookmarkInfo]
    let onSelect: (String) -> Void
    let onCreate: (String) -> Void
    let onDelete: (String) -> Void

    @State private var showingCreate = false
    @State private var newBookmarkName = ""

    var body: some View {
        Menu {
            if !bookmarks.isEmpty {
                ForEach(bookmarks, id: \.name) { bookmark in
                    Button {
                        onSelect(bookmark.name)
                    } label: {
                        HStack {
                            Text(bookmark.name)
                            if bookmark.isTrackingRemote {
                                Image(systemName: "cloud")
                            }
                        }
                    }
                }

                Divider()

                Menu("Delete Bookmark") {
                    ForEach(bookmarks, id: \.name) { bookmark in
                        Button(role: .destructive) {
                            onDelete(bookmark.name)
                        } label: {
                            Text(bookmark.name)
                        }
                    }
                }
            }

            Button("New Bookmark...") {
                newBookmarkName = ""
                showingCreate = true
            }
        } label: {
            HStack(spacing: 4) {
                Image(systemName: "arrow.triangle.branch")
                    .imageScale(.small)
                if let current = bookmarks.first {
                    Text(current.name)
                        .jayjayFont(12, weight: .medium)
                        .lineLimit(1)
                } else {
                    Text("Bookmarks")
                        .jayjayFont(12, weight: .medium)
                }
            }
        }
        .menuStyle(.borderlessButton)
        .fixedSize()
        .popover(isPresented: $showingCreate) {
            VStack(alignment: .leading, spacing: 10) {
                Text("New Bookmark")
                    .jayjayFont(13, weight: .semibold)
                TextField("Bookmark name", text: $newBookmarkName)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(13, design: .monospaced)
                    .frame(width: 220)
                    .onSubmit { submitCreate() }
                HStack {
                    Spacer()
                    Button("Cancel") { showingCreate = false }
                        .keyboardShortcut(.cancelAction)
                    Button("Create") { submitCreate() }
                        .keyboardShortcut(.defaultAction)
                        .disabled(newBookmarkName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }
            .padding(14)
        }
    }

    private func submitCreate() {
        let name = newBookmarkName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty else { return }
        onCreate(name)
        showingCreate = false
    }
}
