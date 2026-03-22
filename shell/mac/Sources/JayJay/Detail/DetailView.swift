import JayJayCore
import SwiftUI

struct DetailView: View {
    let repoPath: String
    let repo: JayJayRepo?
    let detail: ChangeDetail?
    let actions: (any ChangeActions)?
    let onDescribe: (String, String) -> Void
    let reviewStore: ReviewStore

    var body: some View {
        if let detail {
            ChangeDetailView(
                repoPath: repoPath, repo: repo, detail: detail,
                actions: actions, onDescribe: onDescribe,
                reviewStore: reviewStore
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
    let actions: (any ChangeActions)?
    let onDescribe: (String, String) -> Void
    let reviewStore: ReviewStore

    @State private var editingDescription = false
    @State private var descriptionText = ""
    @State private var selectedPath: String?
    @State private var showSplitSheet = false
    @State private var splitPaths: [String] = []
    @State private var splitMessage = ""
    @State private var fileFilter = ""
    @Environment(AppSettings.self) private var appSettings

    private var reviewedPaths: Set<String> {
        reviewStore.reviewedPaths(
            changeId: detail.info.changeId,
            allPaths: detail.diff.map(\.path)
        )
    }

    private var filteredDiff: [DiffHunk] {
        guard !fileFilter.isEmpty else { return detail.diff }
        return detail.diff.filter { $0.path.localizedCaseInsensitiveContains(fileFilter) }
    }

    var body: some View {
        Group {
            if detail.diff.isEmpty {
                emptyState
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
            VStack(alignment: .leading, spacing: 12) {
                Text("Split \(splitPaths.count) \(splitPaths.count == 1 ? "file" : "files") to new change")
                    .jayjayFont(14, weight: .semibold)
                Text(splitPaths.sorted().joined(separator: "\n"))
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary)
                    .lineLimit(10)
                TextField("Description for split change", text: $splitMessage)
                    .textFieldStyle(.roundedBorder)
                HStack {
                    Spacer()
                    Button("Cancel") { showSplitSheet = false }
                        .keyboardShortcut(.cancelAction)
                    Button("Split") {
                        actions?.split(rev: detail.info.changeId, paths: splitPaths, message: splitMessage)
                        showSplitSheet = false
                        splitMessage = ""
                        for p in splitPaths {
                            reviewStore.markUnreviewed(changeId: detail.info.changeId, path: p)
                        }
                        splitPaths = []
                    }
                    .keyboardShortcut(.defaultAction)
                    .disabled(splitMessage.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }
            .padding(20)
            .frame(width: 400)
        }
    }

    private var selectedHunk: DiffHunk? {
        if let selectedPath { return detail.diff.first(where: { $0.path == selectedPath }) ?? detail.diff.first }
        return detail.diff.first
    }

    // MARK: - Empty state

    private var emptyState: some View {
        VStack(alignment: .leading, spacing: 16) {
            headerSection
            descriptionSection
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

    // MARK: - File column

    private var fileColumn: some View {
        VStack(spacing: 0) {
            HStack(spacing: 6) {
                if fileFilter.isEmpty {
                    Text("\(detail.diff.count) files")
                        .jayjayFont(11, weight: .medium)
                        .foregroundStyle(.secondary)
                } else {
                    Text("\(filteredDiff.count) of \(detail.diff.count) files")
                        .jayjayFont(11, weight: .medium)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                if detail.info.isWorkingCopy, !reviewedPaths.isEmpty {
                    Text("\(reviewedPaths.count)/\(detail.diff.count)")
                        .jayjayFont(10, weight: .semibold)
                        .foregroundStyle(reviewedPaths.count == detail.diff.count ? .green : .secondary)

                    Button {
                        splitPaths = Array(reviewedPaths)
                        showSplitSheet = true
                    } label: {
                        Text("Split")
                            .jayjayFont(10, weight: .semibold)
                    }
                    .controlSize(.mini)
                    .help("Split \(reviewedPaths.count) checked files to a new change")
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)

            HStack(spacing: 4) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                    .jayjayFont(11)
                TextField("Filter files\u{2026}", text: $fileFilter)
                    .textFieldStyle(.plain)
                    .jayjayFont(12)
                if !fileFilter.isEmpty {
                    Button {
                        fileFilter = ""
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.secondary)
                            .jayjayFont(11)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 5)
            .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 6, style: .continuous))
            .padding(.horizontal, 8)
            .padding(.bottom, 6)

            Divider()
            ScrollView {
                if appSettings.treeFileList {
                    treeContent
                } else {
                    flatContent
                }
            }
            .focusable()
            .focusEffectDisabled()
            .onKeyPress(.space) {
                guard detail.info.isWorkingCopy, let path = selectedPath else { return .ignored }
                toggleReview(path)
                if reviewedPaths.contains(path),
                   let next = filteredDiff.first(where: { !reviewedPaths.contains($0.path) }) {
                    selectedPath = next.path
                }
                return .handled
            }
            .onKeyPress(.upArrow) {
                guard let cur = selectedPath, let i = filteredDiff.firstIndex(where: { $0.path == cur }),
                      i > 0 else { return .ignored }
                selectedPath = filteredDiff[i - 1].path
                return .handled
            }
            .onKeyPress(.downArrow) {
                guard let cur = selectedPath, let i = filteredDiff.firstIndex(where: { $0.path == cur }),
                      i < filteredDiff.count - 1 else { return .ignored }
                selectedPath = filteredDiff[i + 1].path
                return .handled
            }
        }
        .frame(maxHeight: .infinity)
        .background(Color.primary.opacity(0.02))
    }

    private var flatContent: some View {
        LazyVStack(alignment: .leading, spacing: 6) {
            ForEach(filteredDiff, id: \.path) { hunk in
                fileRowView(hunk: hunk)
            }
        }
        .padding(10)
    }

    private var treeContent: some View {
        let entries = filteredDiff.buildTree()
        return LazyVStack(alignment: .leading, spacing: 2) {
            ForEach(entries) { entry in
                if let hunk = entry.hunk {
                    fileRowView(hunk: hunk)
                        .padding(.leading, CGFloat(entry.depth) * 16)
                } else {
                    HStack(spacing: 5) {
                        Image(systemName: "folder.fill")
                            .jayjayFont(10)
                            .foregroundStyle(.secondary.opacity(0.6))
                        Text(entry.name)
                            .jayjayFont(11, weight: .medium)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.leading, CGFloat(entry.depth) * 16 + 10)
                    .padding(.vertical, 4)
                }
            }
        }
        .padding(.vertical, 6)
        .padding(.horizontal, 4)
    }

    private func fileRowView(hunk: DiffHunk) -> some View {
        FileRow(
            hunk: hunk,
            isSelected: selectedHunk?.path == hunk.path,
            showReview: detail.info.isWorkingCopy,
            isReviewed: reviewedPaths.contains(hunk.path),
            onToggleReview: { toggleReview(hunk.path) }
        )
        .contentShape(Rectangle())
        .onTapGesture { selectedPath = hunk.path }
        .contextMenu {
            Button("Open in \(appSettings.externalEditor.title)") {
                appSettings.openInEditor(filePath: hunk.path, repoPath: repoPath)
            }
            Button("Show in Finder") { showInFinder(hunk.path) }
            Button("Copy Path") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(hunk.path, forType: .string)
            }
            Divider()
            if detail.info.isWorkingCopy {
                Button(reviewedPaths.contains(hunk.path) ? "Mark as Unreviewed" : "Mark as Reviewed") {
                    toggleReview(hunk.path)
                }
                Divider()
            }
            Button("Split to New Change") {
                splitPaths = [hunk.path]
                showSplitSheet = true
            }
            if !detail.info.isWorkingCopy {
                Button("Move to Working Copy") {
                    actions?.moveToWorkingCopy(rev: detail.info.changeId, paths: [hunk.path])
                }
            }
            Button("Restore to Parent") { actions?.restoreFiles(rev: detail.info.changeId, paths: [hunk.path]) }
            if detail.info.isWorkingCopy {
                Button("Delete from Disk", role: .destructive) { actions?.deleteFiles(paths: [hunk.path]) }
            }
            Divider()
            Button("Ignore & Untrack") { actions?.ignoreAndUntrack(paths: [hunk.path]) }
        }
    }

    // MARK: - Preview column

    private var previewColumn: some View {
        VStack(alignment: .leading, spacing: 0) {
            VStack(alignment: .leading, spacing: 12) {
                headerSection
                descriptionSection
            }
            .padding(.horizontal, 18)
            .padding(.top, 14)
            .padding(.bottom, 8)

            Divider()

            if let hunk = selectedHunk {
                DiffSection(hunk: hunk, rev: detail.info.changeId, repo: repo)
                    .padding(.horizontal, 18)
                    .padding(.top, 10)
                    .padding(.bottom, 6)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
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

    // MARK: - Header & description

    private var headerSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            LabeledRow("Change", value: detail.info.changeId)
            LabeledRow("Commit", value: String(detail.info.commitId.prefix(12)))
            LabeledRow("Author", value: "\(detail.info.author) <\(detail.info.email)>")
            LabeledRow("Date", value: formatTimestamp(detail.info.timestampMillis))
            if !detail.info.parents.isEmpty {
                LabeledRow("Parents", value: detail.info.parents.map { String($0.prefix(12)) }.joined(separator: ", "))
            }
            if !detail.info.bookmarks.isEmpty {
                HStack(spacing: 4) {
                    Text("Bookmarks").jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
                    ForEach(detail.info.bookmarks, id: \.self) { name in
                        Text(name).jayjayFont(11, design: .monospaced)
                            .padding(.horizontal, 6).padding(.vertical, 2)
                            .background(.tint.opacity(0.15), in: .capsule)
                    }
                }
            }
        }
    }

    private var descriptionSection: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("Description").jayjayFont(17, weight: .semibold)
                Spacer()
                if editingDescription {
                    Button("Save") { onDescribe(detail.info.changeId, descriptionText)
                        editingDescription = false
                    }
                    .keyboardShortcut("s")
                    Button("Cancel") { descriptionText = detail.info.description
                        editingDescription = false
                    }
                    .keyboardShortcut(.cancelAction)
                } else {
                    Button("Edit") { editingDescription = true }
                }
            }
            if editingDescription {
                TextEditor(text: $descriptionText)
                    .jayjayFont(13, design: .monospaced)
                    .frame(minHeight: 80).border(.separator)
            } else if detail.info.description.isEmpty {
                Text("(no description)").foregroundStyle(.tertiary).italic()
            } else {
                Text(detail.info.description).jayjayFont(13, design: .monospaced).textSelection(.enabled)
            }
        }
    }

    // MARK: - Helpers

    private func resetState() {
        descriptionText = detail.info.description
        editingDescription = false
        selectedPath = detail.diff.first?.path
        fileFilter = ""
        DiffSection.clearCache()
    }

    private func toggleReview(_ path: String) {
        reviewStore.toggleReviewed(changeId: detail.info.changeId, path: path)
    }

    private func showInFinder(_ path: String) {
        NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: repoPath).appendingPathComponent(path)])
    }

    private func formatTimestamp(_ millis: Int64) -> String {
        Date(timeIntervalSince1970: Double(millis) / 1000.0).formatted(.dateTime.year().month().day().hour().minute())
    }
}
