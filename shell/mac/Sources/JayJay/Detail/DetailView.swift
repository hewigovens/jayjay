import JayJayCore
import SwiftUI

struct DetailView: View {
    let repoPath: String
    let repo: JayJayRepo?
    let detail: ChangeDetail?
    let actions: (any ChangeActions & DAGActions)?
    let onDescribe: (String, String) -> Void
    let reviewStore: ReviewStore
    let diffStore: DiffStore
    var compareFromId: String?
    var onClearCompare: (() -> Void)?
    var onRevealChangeInDag: ((String) -> Void)?
    @Binding var activePane: ActivePane
    var evologEntries: [EvologEntry]?
    var evologRev: String?
    var onDismissEvolog: (() -> Void)?

    var body: some View {
        if let entries = evologEntries, let rev = evologRev {
            EvologView(
                entries: entries,
                changeId: rev,
                repo: repo,
                diffStore: diffStore,
                onDismiss: { onDismissEvolog?() }
            )
            .id(rev)
        } else if let detail {
            ChangeDetailView(
                repoPath: repoPath, repo: repo, detail: detail,
                actions: actions, onDescribe: onDescribe,
                reviewStore: reviewStore, diffStore: diffStore,
                compareFromId: compareFromId,
                onClearCompare: onClearCompare,
                onRevealChangeInDag: onRevealChangeInDag,
                activePane: $activePane
            )
            .id(detail.info.changeId)
        } else {
            ContentUnavailableView(
                "Select a Change", systemImage: "doc.text",
                description: Text("Choose a change from the list to see its details.")
            )
        }
    }
}

struct ChangeDetailView: View {
    let repoPath: String
    let repo: JayJayRepo?
    let detail: ChangeDetail
    let actions: (any ChangeActions & DAGActions)?
    let onDescribe: (String, String) -> Void
    let reviewStore: ReviewStore
    let diffStore: DiffStore
    var compareFromId: String?
    var onClearCompare: (() -> Void)?
    var onRevealChangeInDag: ((String) -> Void)?
    @Binding var activePane: ActivePane

    var isCompareMode: Bool {
        compareFromId != nil
    }

    @State var editingDescription = false
    @State var descriptionText = ""
    @State var selectedPath: String?
    @State var selectedPaths: Set<String> = []
    @State var fileSelectionAnchorPath: String?
    @State var showSplitSheet = false
    @State var splitPaths: [String] = []
    @State var splitMessage = ""
    @State var splitParallel = false
    @State var showFileFilter = false
    @State var fileFilter = ""
    @State var hideReviewedFiles = false
    @State var diffStats: DiffStats?
    @State var paneMode: DetailPaneMode = .files
    @State var conflictedPaths: Set<String> = []
    @State var trackedGitLfsPaths: Set<String> = []
    @State var reviewedPaths: Set<String> = []
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
        if hideReviewedFiles {
            result = result.filter { !reviewedPaths.contains($0.path) }
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
                    onDone: { paneMode = .files }
                )
            } else {
                HSplitView {
                    fileColumn
                        .frame(minWidth: 220, idealWidth: 260, maxWidth: 320)
                    previewColumn
                        .frame(minWidth: 420)
                }
            }
        }
        .onAppear { resetState() }
        .onChange(of: detail.info.commitId) { _, _ in
            resetState(preservingFileContext: detail.info.isWorkingCopy)
        }
        .sheet(isPresented: $showSplitSheet) {
            SheetContainer(
                title: "Split \(splitPaths.count) \(splitPaths.count == 1 ? "file" : "files") to new change",
                subtitle: splitPaths.sorted().joined(separator: "\n"),
                cancelLabel: "Cancel",
                confirmLabel: "Split",
                confirmDisabled: splitMessage.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
                onCancel: { showSplitSheet = false },
                onConfirm: {
                    actions?.split(
                        rev: detail.info.changeId, paths: splitPaths,
                        message: splitMessage, parallel: splitParallel
                    )
                    showSplitSheet = false
                    splitMessage = ""
                    splitParallel = false
                    for p in splitPaths {
                        reviewStore.markUnreviewed(changeId: detail.info.changeId, path: p)
                    }
                    splitPaths = []
                },
                content: {
                    TextField("Description for split change", text: $splitMessage)
                        .textFieldStyle(.roundedBorder)
                        .onSubmit {
                            guard !splitMessage.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
                        }
                    Toggle("Parallel split", isOn: $splitParallel)
                        .jayjayFont(12)
                }
            )
            .frame(width: 400)
        }
    }

    var selectedHunk: DiffHunk? {
        if let selectedPath {
            return filteredDiff.first(where: { $0.path == selectedPath }) ?? filteredDiff.first
        }
        return filteredDiff.first
    }

    // MARK: - Empty state

    private var emptyState: some View {
        VStack(alignment: .leading, spacing: 16) {
            headerSection
            descriptionSection
            Divider()
            if visibleDiff.isEmpty, hiddenDiffCount > 0 {
                ContentUnavailableView(
                    hiddenOnlyStateTitle,
                    systemImage: hiddenOnlyStateIcon,
                    description: Text(hiddenOnlyStateDescription)
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ContentUnavailableView(
                    "No Files Changed",
                    systemImage: "doc.badge.minus",
                    description: Text("This revision does not modify any tracked files.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .padding(18)
    }

    // MARK: - Preview column

    var previewColumn: some View {
        VStack(alignment: .leading, spacing: 0) {
            if isCompareMode {
                compareBanner
            }
            VStack(alignment: .leading, spacing: 12) {
                if !isCompareMode {
                    headerSection
                    descriptionSection
                }
            }
            .padding(.horizontal, 18)
            .padding(.top, isCompareMode ? 4 : 14)
            .padding(.bottom, 8)
            .zIndex(1)

            Divider()

            if case let .annotate(lines, path) = paneMode {
                AnnotateView(
                    lines: lines, path: path, repo: repo,
                    onSelectChange: { changeId in
                        paneMode = .files
                        if let onRevealChangeInDag {
                            onRevealChangeInDag(changeId)
                        } else {
                            actions?.select(changeId: changeId)
                        }
                    },
                    onDismiss: { paneMode = .files }
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if case let .fileHistory(history, path) = paneMode {
                FileHistoryView(
                    history: history, path: path,
                    onSelectChange: { changeId in
                        paneMode = .files
                        if let onRevealChangeInDag {
                            onRevealChangeInDag(changeId)
                        } else {
                            actions?.select(changeId: changeId)
                        }
                    },
                    onDismiss: { paneMode = .files }
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let hunk = selectedHunk {
                VStack(spacing: 0) {
                    if conflictedPaths.contains(hunk.path) {
                        conflictBar(path: hunk.path)
                    }
                    DiffSection(
                        hunk: hunk,
                        rev: detail.info.changeId,
                        repo: repo,
                        actions: actions,
                        isWorkingCopy: detail.info.isWorkingCopy,
                        diffStore: diffStore,
                        reviewStore: reviewStore,
                        onOpenDiffEdit: {
                            paneMode = .diffEdit
                        },
                        onReviewStateChanged: { refreshReviewedPaths() },
                        compareFromRev: compareFromId
                    )
                    // Rebuild DiffSection on commit-id change so Abandon-Selected-Lines refreshes @State fileDiff.
                    .id("\(detail.info.commitId)|\(hunk.path)")
                    .padding(.horizontal, 18)
                    .padding(.top, 10)
                    .padding(.bottom, 6)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                }
            } else {
                Spacer()
                ContentUnavailableView(
                    "Select a File",
                    systemImage: "doc.text.magnifyingglass",
                    description: Text("Choose a file to inspect.")
                )
                .frame(maxWidth: .infinity)
                Spacer()
            }
        }
        .frame(maxHeight: .infinity)
    }

    private var hiddenOnlyStateTitle: String {
        if hiddenGitLfsCount > 0, hiddenSubmoduleCount > 0 {
            return "Only Git-managed Changes Hidden"
        }
        if hiddenGitLfsCount > 0 {
            return "Only Git LFS-backed Files Hidden"
        }
        return "Only Git Submodule Changes Hidden"
    }

    private var hiddenOnlyStateIcon: String {
        if hiddenGitLfsCount > 0, hiddenSubmoduleCount > 0 {
            return "externaldrive.badge.questionmark"
        }
        if hiddenGitLfsCount > 0 {
            return "externaldrive.badge.questionmark"
        }
        return "square.stack.3d.up.slash"
    }

    private var hiddenOnlyStateDescription: String {
        if hiddenGitLfsCount > 0, hiddenSubmoduleCount > 0 {
            return "This revision only changes Git LFS-backed files and submodules, and the current diff settings hide them."
        }
        if hiddenGitLfsCount > 0 {
            return "This revision only changes Git LFS-backed files, and the current diff setting hides them."
        }
        return "This revision only changes Git submodules, and the current settings hide them."
    }
}
