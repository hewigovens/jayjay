import JayJayCore
import JayJayDiffUI
import SwiftUI

struct DiffEditView: View {
    @State var session: DiffEditSession

    init(
        detail: ChangeDetail,
        repo: JayJayRepo?,
        diffStore: DiffStore,
        actions: (any ChangeActions)?,
        diffStats: DiffStats?,
        settings: AppSettings,
        onDone: @escaping () -> Void
    ) {
        _session = State(initialValue: DiffEditSession(
            detail: detail,
            repo: repo,
            diffStore: diffStore,
            actions: actions,
            diffStats: diffStats,
            settings: settings,
            onDone: onDone
        ))
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 14) {
                        if hasUnsupportedFiles {
                            unsupportedNotice
                        }
                        ForEach(session.detail.diff, id: \.path) { hunk in
                            DiffEditFileSection(
                                hunk: hunk,
                                rev: session.sessionCommit,
                                commitId: session.sessionCommit,
                                repo: session.repo,
                                diffStore: session.diffStore,
                                selectedChangedLines: session.selectedChangedLinesByPath[hunk.path] ?? [],
                                stats: session.fileStats[hunk.path],
                                isCollapsed: session.collapsedPaths.contains(hunk.path),
                                isFocused: session.focusedPath == hunk.path,
                                onToggleCollapse: {
                                    session.focusedPath = hunk.path
                                    session.toggleCollapse(path: hunk.path)
                                },
                                onToggleFile: { session.toggleFileSelection(path: hunk.path) },
                                onSelectFile: { session.selectFile(path: hunk.path) },
                                onToggleLine: { session.toggleLineSelection(path: hunk.path, lineNumber: $0) },
                                onSelectHunk: { session.selectHunk(path: hunk.path, range: $0) },
                                onLoaded: { session.fileLoaded(path: hunk.path, loaded: $0) }
                            )
                            .id(hunk.path)
                        }
                    }
                    .padding(18)
                }
                .onChange(of: session.focusedPath) { _, path in
                    guard let path else { return }
                    withAnimation(.easeInOut(duration: 0.18)) {
                        proxy.scrollTo(path, anchor: .top)
                    }
                }
            }
        }
        .background(
            KeyDownMonitor(ignoresReadOnlyText: true, onKeyDown: { session.handleKey($0) })
                .frame(width: 0, height: 0)
                .allowsHitTesting(false)
        )
        .safeAreaInset(edge: .bottom) {
            actionBar
        }
        .alert("Nothing Selected", isPresented: Bindable(session).showEmptySelectionAlert) {
            Button("OK", role: .cancel) {}
        } message: {
            Text("Select at least one file, hunk, or line before applying diff edit.")
        }
        .alert(
            "Couldn't Load All Files",
            isPresented: Binding(
                get: { session.applyLoadFailurePath != nil },
                set: {
                    if !$0 {
                        session.applyLoadFailurePath = nil
                    }
                }
            )
        ) {
            Button("OK", role: .cancel) {}
        } message: {
            Text("Failed to load \(session.applyLoadFailurePath ?? ""). Done was not applied — try again.")
        }
        .alert(
            "Diff Changed",
            isPresented: Binding(
                get: { session.applyStalePath != nil },
                set: {
                    if !$0 {
                        session.applyStalePath = nil
                    }
                }
            )
        ) {
            Button("OK", role: .cancel) {}
        } message: {
            Text("\(session.applyStalePath ?? "") changed after its selection was rendered. Done was not applied — refresh and try again.")
        }
        .task(id: "\(session.sessionCommit)|\(session.settings.ignoreWhitespace)") {
            await session.loadFileStats()
        }
        .onChange(of: session.settings.ignoreWhitespace) { _, _ in
            session.whitespaceModeChanged()
        }
        .onDisappear {
            session.cancelTasks()
        }
    }

    private var header: some View {
        HStack(spacing: 12) {
            Label("Diff Edit", systemImage: "slider.horizontal.3")
                .jayjayFont(15, weight: .semibold)
            Text(String(session.detailRevision.prefix(12)))
                .jayjayFont(12, design: .monospaced)
                .foregroundStyle(.secondary)
            Spacer()
            Text(session.selectionSummary)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button {
                session.expandAllFiles()
            } label: {
                Label("Expand All", systemImage: "rectangle.expand.vertical")
            }
            .keyboardShortcut("e", modifiers: [.command, .option])
            .accessibilityIdentifier(AID.DiffEdit.expandAll)
            Button {
                session.collapseAllFiles()
            } label: {
                Label("Collapse All", systemImage: "rectangle.compress.vertical")
            }
            .keyboardShortcut("c", modifiers: [.command, .option])
            .accessibilityIdentifier(AID.DiffEdit.collapseAll)
            Button {
                session.toggleBulkSelection()
            } label: {
                Label(session.selectionToggleTitle, systemImage: session.selectionToggleSystemImage)
            }
            .disabled(session.selectionToggleDisabled)
            .controlSize(.small)
            Button("Cancel", action: session.onDone)
                .keyboardShortcut(.cancelAction)
                .accessibilityIdentifier(AID.DiffEdit.cancel)
        }
        .controlSize(.small)
        .padding(.horizontal, 18)
        .padding(.vertical, 12)
        .background(.background)
    }

    private var unsupportedNotice: some View {
        HStack(spacing: 10) {
            Image(systemName: "info.circle")
                .foregroundStyle(.secondary)
            Text("Projected, renamed, and non-text files can be previewed here but are not editable yet.")
                .jayjayFont(12)
                .foregroundStyle(.secondary)
            Spacer()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private var actionBar: some View {
        VStack(spacing: 10) {
            Divider()
            HStack(spacing: 12) {
                Text(session.selectionSummary)
                    .jayjayFont(12, weight: .medium)
                Spacer()
                if !session.detail.info.isWorkingCopy {
                    TextField("New change description", text: Bindable(session).newChangeMessage)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 260)
                }
                if !session.detail.info.isWorkingCopy {
                    Button("Create New Child Change") { session.apply(.newChild) }
                        .buttonStyle(.borderedProminent)
                    Button("Create Parallel Change") { session.apply(.newParallel) }
                        .buttonStyle(.bordered)
                    Button("Move to Working Copy") { session.apply(.moveToWorkingCopy) }
                        .buttonStyle(.bordered)
                }
                Button("Done") {
                    session.apply(.removeFromSource)
                }
                .buttonStyle(.bordered)
            }
            .disabled(session.isPreparingRemoval)
            .padding(.horizontal, 18)
            .padding(.bottom, 12)
        }
        .background(.bar)
    }

    private var hasUnsupportedFiles: Bool {
        session.detail.diff.contains { hunk in
            hunk.projection != nil
                || hunk.hunkType == .renamed
                || !DiffPlaceholder.isEditableText(hunk.oldContent)
                || !DiffPlaceholder.isEditableText(hunk.newContent)
        }
    }
}
