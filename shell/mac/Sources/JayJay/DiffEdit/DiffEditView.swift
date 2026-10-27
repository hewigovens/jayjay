import JayJayCore
import JayJayDiffUI
import SwiftUI

struct DiffEditView: View {
    @Environment(\.colorScheme) private var colorScheme
    @State var session: DiffEditViewModel

    init(
        detail: ChangeDetail,
        repo: JayJayRepo?,
        diffStore: DiffStore,
        actions: (any ChangeActions)?,
        diffStats: DiffStats?,
        settings: AppSettings,
        onDone: @escaping () -> Void
    ) {
        _session = State(initialValue: DiffEditViewModel(
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
            DiffEditHeader(session: session)
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
                                selection: session.fileSelection(for: hunk.path),
                                stats: session.fileStats[hunk.path],
                                isCollapsed: session.collapsedPaths.contains(hunk.path),
                                isFocused: session.focusedPath == hunk.path,
                                onToggleCollapse: {
                                    session.focusCard(path: hunk.path)
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
        .background(session.settings.readingBackground(colorScheme))
        .background(
            KeyDownMonitor(yieldsToText: \.isEditable, swallowsUnhandledKeys: true, onKeyDown: { session.handleKey($0) })
                .frame(width: 0, height: 0)
                .allowsHitTesting(false)
        )
        .safeAreaInset(edge: .bottom) {
            DiffEditActionBar(session: session)
        }
        .alert(Localized.Diff.nothingSelected, isPresented: Bindable(session).showEmptySelectionAlert) {
            Button(Localized.Repo.ok, role: .cancel) {}
        } message: {
            Text(Localized.Repo.selectAtLeastOneFileHunk)
        }
        .alert(
            Localized.Diff.couldntLoadAllFiles,
            isPresented: Binding(
                get: { session.applyLoadFailurePath != nil },
                set: {
                    if !$0 {
                        session.applyLoadFailurePath = nil
                    }
                }
            )
        ) {
            Button(Localized.Repo.ok, role: .cancel) {}
        } message: {
            Text(Localized.Diff.editLoadFailed(path: session.applyLoadFailurePath ?? ""))
        }
        .alert(
            Localized.Diff.diffChanged,
            isPresented: Binding(
                get: { session.applyStalePath != nil },
                set: {
                    if !$0 {
                        session.applyStalePath = nil
                    }
                }
            )
        ) {
            Button(Localized.Repo.ok, role: .cancel) {}
        } message: {
            Text(Localized.Diff.editSelectionStale(path: session.applyStalePath ?? ""))
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

    private var unsupportedNotice: some View {
        HStack(spacing: 10) {
            Image(systemName: "info.circle")
                .foregroundStyle(.secondary)
            Text(Localized.Repo.projectedRenamedAndNonTextFiles)
                .jayjayFont(12)
                .foregroundStyle(.secondary)
            Spacer()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private var hasUnsupportedFiles: Bool {
        session.detail.diff.contains { hunk in
            hunk.projection != nil
                || hunk.hunkType.hasSource
                || !DiffPlaceholder.isEditableText(hunk.oldContent)
                || !DiffPlaceholder.isEditableText(hunk.newContent)
        }
    }
}
