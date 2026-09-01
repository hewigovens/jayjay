import JayJayCore
import SwiftUI

struct ChangeDetailView: View {
    let repoPath: String
    let repo: JayJayRepo?
    let detail: ChangeDetail
    let actions: (any ChangeActions & DAGActions)?
    let onDescribe: (String, String) -> Void
    let reviewStore: ReviewStore
    let diffStore: DiffStore
    var compareFromId: String?
    var compareDisplay: CompareDisplay?
    var onClearCompare: (() -> Void)?
    var onReverseCompare: (() -> Void)?
    var onRevealChangeInDag: ((String) -> Void)?
    @Binding var activePane: ActivePane
    var conflictedBookmarkNames: Set<String> = []
    var onInteractionStateChanged: (Bool) -> Void = { _ in }

    var isCompareMode: Bool {
        compareFromId != nil
    }

    var showsReviewControls: Bool {
        detail.info.isWorkingCopy && !isCompareMode
    }

    var detailRevision: String {
        detail.info.selectionRevision
    }

    /// Review store key. Never detailRevision: that is a commit id for divergent changes, while GPUI and CLI/core reconciliation key review state by the change id.
    var reviewChangeId: String {
        detail.info.changeId.id
    }

    @State var editingDescription = false
    @State var descriptionText = ""
    @State var selectedPath: String?
    @State var selectedPaths: Set<String> = []
    @State var fileSelectionAnchorPath: String?
    @State var splitRequest: SplitSheetRequest?
    @State var showFileFilter = false
    @State var fileFilter = ""
    @FocusState var fileFilterFocused: Bool
    @State var hideReviewedFiles = false
    @State var showNotedFilesOnly = false
    @State var diffStats: DiffStats?
    @State var paneMode: DetailPaneMode = .files
    @State private var fileColumnWidth: CGFloat = 260
    @State var paneBeforeDiffEdit: ActivePane?
    @State var conflictedPaths: Set<String> = []
    @State var trackedGitLfsPaths: Set<String> = []
    @State var reviewedPaths: Set<String> = []
    @State var fileRollups: [String: ReviewFileRollup] = [:]
    // Identity-keyed and never cleared on refresh: a working-copy reload must not re-materialize every changed file to rebuild snapshots it already has.
    @State var reviewSnapshots: [ReviewSnapshotKey: ReviewFileSnapshot] = [:]
    @State var resolvedReviewSnapshotKeys: Set<ReviewSnapshotKey> = []
    @State var reviewSnapshotRequestId: UInt64 = 0
    @State var reviewNoteStatuses: [ReviewNoteStatus] = []
    @State var reviewNotesRequestId: UInt64 = 0
    // Lives here, not in DiffSection: the diff view is rebuilt on commit-id changes, and a background snapshot mid-typing would dismiss the editor sheet.
    @State var noteEditor: ReviewNoteEditorState?
    @State var fileEditor: WorkingCopyFileEditorSession?
    @State var fileEditorPreparation: EditorPreparationRequest?
    @State var conflictEditor: ConflictEditorSession?
    @State var conflictEditorPreparation: EditorPreparationRequest?
    @State var activeNoteCountsByPath: [String: Int] = [:]
    @State var diffStatsCommitId: String?
    @Environment(AppSettings.self) var appSettings

    var visibleDiff: [DiffHunk] {
        detail.diff.filter { hunk in
            if appSettings.hideGitLfsDiffs,
               hunk.isGitLfsPlaceholder || trackedGitLfsPaths.contains(hunk.path)
            {
                return false
            }
            if !appSettings.enableGitSubmoduleSupport, hunk.isSubmodulePlaceholder {
                return false
            }
            return true
        }
    }

    var filteredDiff: [DiffHunk] {
        var result = visibleDiff
        if !fileFilter.isEmpty {
            result = result.filter { $0.path.localizedCaseInsensitiveContains(fileFilter) }
        }
        if hideReviewedFiles, showsReviewControls {
            result = result.filter { !reviewedPaths.contains($0.path) }
        }
        if showNotedFilesOnly, showsReviewControls {
            result = result.filter { activeNoteCountsByPath[$0.path] != nil }
        }
        return result
    }

    var hiddenGitLfsCount: Int {
        guard appSettings.hideGitLfsDiffs else { return 0 }
        return detail.diff.filter { hunk in
            hunk.isGitLfsPlaceholder || trackedGitLfsPaths.contains(hunk.path)
        }.count
    }

    var hiddenSubmoduleCount: Int {
        guard !appSettings.enableGitSubmoduleSupport else { return 0 }
        return detail.diff.filter(\.isSubmodulePlaceholder).count
    }

    var hiddenDiffCount: Int {
        hiddenGitLfsCount + hiddenSubmoduleCount
    }

    var body: some View {
        Group {
            if detail.diff.isEmpty || (visibleDiff.isEmpty && hiddenDiffCount > 0) {
                emptyState
            } else if paneMode.isDiffEdit {
                DiffEditView(
                    detail: detail,
                    repo: repo,
                    diffStore: diffStore,
                    actions: actions,
                    diffStats: diffStats,
                    settings: appSettings,
                    onDone: { paneMode = .files }
                )
            } else {
                GeometryReader { geo in
                    let range = PaneLayout.fileColumnRange(detailWidth: geo.size.width)
                    let width = Binding(get: { min(fileColumnWidth, range.upperBound) }, set: { fileColumnWidth = $0 })
                    HStack(spacing: 0) {
                        fileColumn
                            .frame(width: width.wrappedValue)
                            .accessibilityElement(children: .contain)
                            .accessibilityIdentifier(AID.FileList.column)
                        SidebarDivider(
                            position: width,
                            range: range,
                            onEnded: { appSettings.fileColumnWidth = $0 }
                        )
                        .accessibilityElement()
                        .accessibilityIdentifier(AID.FileList.columnDivider)
                        previewColumn
                            .frame(maxWidth: .infinity)
                    }
                }
            }
        }
        .onAppear {
            fileColumnWidth = appSettings.fileColumnWidth
            resetState()
        }
        // Diff edit must own j/k: the DAG's earlier-installed key monitor would otherwise consume them whenever the DAG was the active pane.
        .onChange(of: paneMode.isDiffEdit) { _, isDiffEdit in
            if isDiffEdit {
                paneBeforeDiffEdit = activePane
                activePane = .fileColumn
            } else {
                // Restore only if the user didn't click another pane while diff edit was open.
                if activePane == .fileColumn, let previous = paneBeforeDiffEdit {
                    activePane = previous
                }
                paneBeforeDiffEdit = nil
            }
        }
        .onChange(of: detail.info.commitId) { _, _ in
            resetState(preservingFileContext: detail.info.isWorkingCopy)
        }
        .onChange(of: reviewStore.resetGeneration) { _, _ in
            refreshReviewState()
        }
        .onChange(of: hasRefreshSensitiveInteraction, initial: true) { _, active in
            onInteractionStateChanged(active)
        }
        .onDisappear {
            onInteractionStateChanged(false)
        }
        .sheet(item: $splitRequest) { request in
            SplitSheetView(
                paths: request.paths,
                onCancel: { splitRequest = nil },
                onConfirm: { message, parallel in
                    confirmSplit(request, message: message, parallel: parallel)
                }
            )
            .frame(width: 400)
        }
        .sheet(item: $noteEditor) { editor in
            ReviewNoteSheet(
                editor: editor,
                onCancel: { noteEditor = nil },
                onSave: { body in saveReviewNote(editor: editor, body: body) }
            )
            .frame(width: 440)
        }
        .sheet(item: $fileEditor) { editor in
            WorkingCopyFileEditorView(
                session: editor,
                onSave: { data, content, completion in
                    actions?.applyWorkingCopyFileEditor(
                        data: data,
                        content: content,
                        completion: completion
                    )
                },
                onDone: { fileEditor = nil }
            )
            .frame(minWidth: 900, idealWidth: 1040, minHeight: 600, idealHeight: 720)
        }
        .sheet(item: $conflictEditor) { editor in
            ConflictEditorView(
                session: editor,
                onSave: { data, content, completion in
                    actions?.applyConflictEditor(
                        rev: editor.target.rev,
                        data: data,
                        content: content,
                        completion: completion
                    )
                },
                onDone: { conflictEditor = nil }
            )
            .frame(minWidth: 1040, idealWidth: 1180, minHeight: 720, idealHeight: 820)
        }
    }

    private var hasRefreshSensitiveInteraction: Bool {
        editingDescription
            || splitRequest != nil
            || paneMode.isDiffEdit
            || noteEditor != nil
            || fileEditor != nil
            || fileEditorPreparation != nil
            || conflictEditor != nil
            || conflictEditorPreparation != nil
    }

    private func confirmSplit(_ request: SplitSheetRequest, message: String, parallel: Bool) {
        actions?.split(
            rev: detailRevision, paths: request.paths,
            message: message, parallel: parallel
        )
        splitRequest = nil
        for path in request.paths {
            reviewStore.markUnreviewed(changeId: reviewChangeId, path: path)
        }
    }

    var selectedHunk: DiffHunk? {
        if let selectedPath {
            return filteredDiff.first(where: { $0.path == selectedPath }) ?? filteredDiff.first
        }
        return filteredDiff.first
    }
}
