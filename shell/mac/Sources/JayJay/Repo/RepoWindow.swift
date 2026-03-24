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
                VStack(spacing: 16) {
                    Image(systemName: "exclamationmark.triangle").font(.system(size: 40)).foregroundStyle(.orange)
                    Text("Failed to open repository").jayjayFont(16, weight: .semibold)
                    Text(err).jayjayFont(12).foregroundStyle(.secondary).textSelection(.enabled)
                        .multilineTextAlignment(.center).frame(maxWidth: 360)
                    if !FileManager.default.fileExists(atPath: "\(repoPath)/.jj") {
                        Button("Initialize with jj git init") {
                            initJJRepo()
                        }
                        .buttonStyle(.borderedProminent)
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ProgressView("Loading repository...")
            }
        }
        .task { openRepo() }
        .navigationTitle(URL(fileURLWithPath: repoPath).lastPathComponent)
        .focusedSceneValue(\.jayjayRepoPath, repoPath)
    }

    private func openRepo() {
        do {
            let model = try RepoViewModel(path: repoPath)
            viewModel = model
            model.refresh()
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

struct RepoContentView: View {
    @Bindable var viewModel: RepoViewModel
    @State var revsetDraft = ""
    @State var showRevsetFilter = false
    @State var sidebarWidth: CGFloat = 360
    @State var bookmarkCreateRev: String?
    @State var bookmarkCreateName = ""
    @State var confirmAbandonRev: String?
    @State var showUndoSheet = false
    let commandPanel = CommandPalettePanel()
    @State var toastMessage: String?
    @Environment(AppSettings.self) var settings
    @Environment(RepoWindowManager.self) var windowManager
    @Environment(\.openSettings) var openSettings
    @Environment(\.colorScheme) var colorScheme

    var body: some View {
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
                        compareFromId: viewModel.compareFromId,
                        onClearCompare: { viewModel.clearCompare() }
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
        .focusedSceneValue(\.jayjayGitFetch) { viewModel.gitFetch() }
        .focusedSceneValue(\.jayjayGitPush) { viewModel.gitPush() }
        .focusedSceneValue(\.jayjaySettings, settings)
        .focusedSceneValue(\.jayjayCommandPalette) { showCommandPalette() }
        .toolbar { toolbarContent }
        .overlay {
            if viewModel
                .isLoading
            { ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity).background(.ultraThinMaterial)
            }
        }
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
        .animation(.easeOut(duration: 0.3), value: toastMessage)
        .alert("Error", isPresented: .init(
            get: { viewModel.error != nil },
            set: { if !$0 { viewModel.error = nil } }
        )) {
            Button("OK") { viewModel.error = nil }
        } message: { Text(viewModel.error ?? "") }
        .onChange(of: viewModel.info) { _, msg in
            guard let msg, !msg.isEmpty else { return }
            showToast(msg)
            viewModel.info = nil
        }
        .sheet(isPresented: .init(get: { bookmarkCreateRev != nil }, set: { if !$0 { bookmarkCreateRev = nil } })) {
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
        toastMessage = message
        Task {
            try? await Task.sleep(for: .seconds(2))
            toastMessage = nil
        }
    }

    // MARK: - Command Palette

    private func showCommandPalette() {
        var items: [CommandPaletteItem] = []
        let sel = viewModel.selectedChangeId

        // View
        items.append(CommandPaletteItem(title: "Refresh", icon: "arrow.triangle.2.circlepath", category: "View") {
            viewModel.refresh()
        })
        items.append(CommandPaletteItem(
            title: "Toggle Side-by-Side Diff", icon: "rectangle.split.2x1", category: "View"
        ) { settings.sideBySideDiff.toggle() })
        items.append(CommandPaletteItem(
            title: "Toggle Tree View", icon: "list.bullet.indent", category: "View"
        ) { settings.treeFileList.toggle() })
        items.append(CommandPaletteItem(
            title: "Toggle Ignore Whitespace", icon: "text.alignleft", category: "View"
        ) { settings.ignoreWhitespace.toggle() })

        // Revset presets
        for (label, revset) in [
            ("Show All", "all()"), ("Show Mine", "mine()"),
            ("Show Bookmarks", "bookmarks()"), ("Show Conflicts", "conflict()"),
        ] {
            items.append(CommandPaletteItem(title: label, icon: "line.3.horizontal.decrease.circle", category: "Filter") {
                revsetDraft = revset
                applyRevset()
            })
        }

        // Git
        items.append(CommandPaletteItem(title: "Git Fetch", icon: "arrow.down.circle", category: "Git") {
            viewModel.gitFetch()
        })
        items.append(CommandPaletteItem(title: "Git Push", icon: "arrow.up.circle", category: "Git") {
            viewModel.gitPush(bookmark: "")
        })

        // Change operations (require selection)
        if let sel {
            items.append(CommandPaletteItem(
                title: "New Child Change", icon: "plus.circle", category: "Change"
            ) { viewModel.newChange(parent: sel) })
            items.append(CommandPaletteItem(
                title: "Edit (Switch To)", icon: "pencil.circle", category: "Change"
            ) { viewModel.edit(rev: sel) })
            items.append(CommandPaletteItem(
                title: "Squash into Parent", icon: "arrow.down.left.circle", category: "Change"
            ) { viewModel.squash(rev: sel) })
            items.append(CommandPaletteItem(
                title: "Duplicate", icon: "doc.on.doc", category: "Change"
            ) { viewModel.duplicate(rev: sel) })
            items.append(CommandPaletteItem(
                title: "Cherry-pick (Graft)", icon: "arrow.triangle.branch", category: "Change"
            ) { viewModel.graft(rev: sel) })
            items.append(CommandPaletteItem(
                title: "Absorb into Ancestors", icon: "arrow.merge", category: "Change"
            ) { viewModel.absorb(rev: sel) })
            items.append(CommandPaletteItem(
                title: "Back Out Change", icon: "arrow.uturn.backward.circle", category: "Change"
            ) { viewModel.backout(rev: sel) })
            items.append(CommandPaletteItem(
                title: "Abandon", icon: "trash", category: "Change"
            ) { requestAbandon(sel) })
            items.append(CommandPaletteItem(
                title: "Create Bookmark Here", icon: "bookmark", category: "Change"
            ) { bookmarkCreateRev = sel; bookmarkCreateName = "" })
        }

        // Tools
        items.append(CommandPaletteItem(title: "Show in Finder", icon: "folder", category: "Tools") {
            RepositoryActions.showInFinder(repoPath: viewModel.repoPath)
        })
        items.append(CommandPaletteItem(
            title: "Open in \(settings.externalEditor.title)", icon: "curlybraces", category: "Tools"
        ) { settings.openInEditor(filePath: ".", repoPath: viewModel.repoPath) })
        items.append(CommandPaletteItem(
            title: "Open in \(settings.terminal.title)", icon: "terminal", category: "Tools"
        ) { settings.openInTerminal(at: viewModel.repoPath) })

        // Repository
        items.append(CommandPaletteItem(
            title: "Undo (Operation Log)", icon: "arrow.uturn.backward", category: "Repository"
        ) { showUndo() })

        // App
        items.append(CommandPaletteItem(title: "Settings", icon: "gearshape", category: "App") { openSettings() })

        commandPanel.show(items: items, repoPath: viewModel.repoPath)
    }

    @ToolbarContentBuilder
    var toolbarContent: some ToolbarContent {
        ToolbarItemGroup(placement: .navigation) {
            BookmarkPicker(
                bookmarks: viewModel.bookmarks,
                actions: viewModel,
                onSelect: { revsetDraft = $0
                    applyRevset()
                }
            )
            Button { showRevsetFilter.toggle() } label: {
                Label("Filter", systemImage: "line.3.horizontal.decrease.circle")
            }.help("Filter by revset")
            Button { viewModel.refresh() } label: {
                ZStack(alignment: .topTrailing) {
                    Label("Refresh", systemImage: "arrow.triangle.2.circlepath")
                    if viewModel.hasWorkingCopyChanges {
                        Circle().fill(.orange).frame(width: 6, height: 6).offset(x: 2, y: -2)
                    }
                }
            }.keyboardShortcut("r")
                .help(viewModel.hasWorkingCopyChanges ? "Files changed — click to refresh (⌘R)" : "Refresh (⌘R)")
            Button { viewModel.gitFetch() } label: {
                Label("Fetch", systemImage: "arrow.down.circle")
            }.help("Git Fetch")
            Button { viewModel.gitPush(bookmark: "") } label: {
                Label("Push", systemImage: "arrow.up.circle")
            }.help("Git Push")
        }

        ToolbarItemGroup(placement: .primaryAction) {
            Button { openSettings() } label: {
                Label("Settings", systemImage: "gearshape")
            }.help("Settings")
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
