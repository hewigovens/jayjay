import JayJayCore
import SwiftUI

struct DetailView: View {
    let repoPath: String
    let repo: JayJayRepo?
    let detail: ChangeDetail?
    let actions: (any ChangeActions & DAGActions)?
    let onDescribe: (String, String) -> Void
    let reviewStore: ReviewStore
    var compareFromId: String?
    var onClearCompare: (() -> Void)?

    var body: some View {
        if let detail {
            ChangeDetailView(
                repoPath: repoPath, repo: repo, detail: detail,
                actions: actions, onDescribe: onDescribe,
                reviewStore: reviewStore,
                compareFromId: compareFromId,
                onClearCompare: onClearCompare
            )
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
    var compareFromId: String?
    var onClearCompare: (() -> Void)?

    var isCompareMode: Bool {
        compareFromId != nil
    }

    @State var editingDescription = false
    @State var descriptionText = ""
    @State var selectedPath: String?
    @State var selectedPaths: Set<String> = []
    @State var fileSelectionAnchorPath: String?
    @FocusState var fileColumnFocused: Bool
    @State var showSplitSheet = false
    @State var splitPaths: [String] = []
    @State var splitMessage = ""
    @State var splitParallel = false
    @State var showFileFilter = false
    @State var fileFilter = ""
    @State var diffStats: DiffStats?
    @State var annotateLines: [AnnotationLine]?
    @State var annotatePath: String?
    @State var fileHistory: [ChangeInfo]?
    @State var fileHistoryPath: String?
    @State var conflictedPaths: Set<String> = []
    @State var isDiffEditMode = false
    @Environment(AppSettings.self) var appSettings

    var reviewedPaths: Set<String> {
        reviewStore.reviewedPaths(
            changeId: detail.info.changeId,
            allPaths: detail.diff.map(\.path)
        )
    }

    var filteredDiff: [DiffHunk] {
        guard !fileFilter.isEmpty else { return detail.diff }
        return detail.diff.filter { $0.path.localizedCaseInsensitiveContains(fileFilter) }
    }

    var body: some View {
        Group {
            if detail.diff.isEmpty {
                emptyState
            } else if isDiffEditMode {
                DiffEditView(
                    detail: detail,
                    repo: repo,
                    actions: actions,
                    onDone: { isDiffEditMode = false }
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
        .onChange(of: detail.info.commitId) { resetState() }
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
        if let selectedPath { return detail.diff.first(where: { $0.path == selectedPath }) ?? detail.diff.first }
        return detail.diff.first
    }

    // MARK: - Empty state

    private var emptyState: some View {
        VStack(alignment: .leading, spacing: 16) {
            headerSection
            descriptionSection
            detailActionsSection
            Divider()
            ContentUnavailableView(
                "No Files Changed",
                systemImage: "doc.badge.minus",
                description: Text("This revision does not modify any tracked files.")
            )
            .frame(maxWidth: .infinity, maxHeight: .infinity)
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
                    detailActionsSection
                }
            }
            .padding(.horizontal, 18)
            .padding(.top, isCompareMode ? 4 : 14)
            .padding(.bottom, 8)

            Divider()

            if let lines = annotateLines, let path = annotatePath {
                AnnotateView(
                    lines: lines, path: path, repo: repo,
                    onSelectChange: { changeId in
                        annotatePath = nil
                        annotateLines = nil
                        actions?.select(changeId: changeId)
                    },
                    onDismiss: { annotatePath = nil
                        annotateLines = nil
                    }
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let history = fileHistory, let path = fileHistoryPath {
                FileHistoryView(
                    history: history, path: path,
                    onSelectChange: { changeId in
                        fileHistoryPath = nil
                        fileHistory = nil
                        actions?.select(changeId: changeId)
                    },
                    onDismiss: { fileHistoryPath = nil
                        fileHistory = nil
                    }
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
                        onOpenDiffEdit: {
                            isDiffEditMode = true
                        },
                        compareFromRev: compareFromId
                    )
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
}
