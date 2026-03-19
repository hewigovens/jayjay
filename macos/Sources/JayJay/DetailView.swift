import SwiftUI
import JayJayBindings

struct DetailView: View {
    let repoPath: String
    let detail: ChangeDetail?
    let onDescribe: (String, String) -> Void
    let onRestoreFiles: (String, [String]) -> Void
    var onIgnoreAndUntrack: (([String]) -> Void)?
    var onSplit: ((String, [String]) -> Void)?

    var body: some View {
        if let detail = detail {
            ChangeDetailView(repoPath: repoPath, detail: detail, onDescribe: onDescribe, onRestoreFiles: onRestoreFiles, onIgnoreAndUntrack: onIgnoreAndUntrack, onSplit: onSplit)
        } else {
            ContentUnavailableView(
                "Select a Change",
                systemImage: "doc.text",
                description: Text("Choose a change from the list to see its details.")
            )
        }
    }
}

struct ChangeDetailView: View {
    let repoPath: String
    let detail: ChangeDetail
    let onDescribe: (String, String) -> Void
    let onRestoreFiles: (String, [String]) -> Void
    var onIgnoreAndUntrack: (([String]) -> Void)?
    var onSplit: ((String, [String]) -> Void)?

    @State private var editingDescription = false
    @State private var descriptionText = ""
    @State private var selectedPath: String?

    var body: some View {
        HSplitView {
            fileColumn
                .frame(minWidth: 220, idealWidth: 260, maxWidth: 320)

            previewColumn
                .frame(minWidth: 420)
        }
        .onAppear {
            resetState()
        }
        .onChange(of: detail.info.commitId) {
            resetState()
        }
    }

    private var selectedHunk: DiffHunk? {
        if let selectedPath {
            return detail.diff.first(where: { $0.path == selectedPath }) ?? detail.diff.first
        }
        return detail.diff.first
    }

    private var fileColumn: some View {
        Group {
            if detail.diff.isEmpty {
                ContentUnavailableView(
                    "No Files Changed",
                    systemImage: "doc.badge.minus",
                    description: Text("This revision does not modify any tracked files.")
                )
            } else {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 8) {
                        ForEach(detail.diff, id: \.path) { hunk in
                            FileRow(
                                hunk: hunk,
                                isSelected: selectedHunk?.path == hunk.path
                            )
                            .contentShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
                            .onTapGesture {
                                selectedPath = hunk.path
                            }
                            .contextMenu {
                                Button("Show in Finder") {
                                    showInFinder(hunk.path)
                                }
                                Button("Copy Path") {
                                    NSPasteboard.general.clearContents()
                                    NSPasteboard.general.setString(hunk.path, forType: .string)
                                }
                                Divider()
                                Button("Split to New Change") {
                                    onSplit?(detail.info.changeId, [hunk.path])
                                }
                                Button("Restore to Parent") {
                                    onRestoreFiles(detail.info.changeId, [hunk.path])
                                }
                                Divider()
                                Button("Ignore & Untrack") {
                                    onIgnoreAndUntrack?([hunk.path])
                                }
                            }
                        }
                    }
                    .padding(12)
                }
            }
        }
        .background(Color.primary.opacity(0.02))
    }

    private var previewColumn: some View {
        VStack(alignment: .leading, spacing: 0) {
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    headerSection
                    descriptionSection
                }
                .padding(18)
                .padding(.bottom, 0)
            }
            .frame(maxHeight: 220)

            Divider()

            if let hunk = selectedHunk {
                previewSection(for: hunk)
                    .padding(.horizontal, 18)
                    .padding(.top, 12)
                    .padding(.bottom, 6)
            } else {
                ContentUnavailableView(
                    "Select a File",
                    systemImage: "doc.text.magnifyingglass",
                    description: Text("Choose a file in this change to inspect its contents.")
                )
                .frame(maxHeight: .infinity)
            }
        }
    }

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
                    Text("Bookmarks")
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                        .frame(width: 70, alignment: .trailing)
                    ForEach(detail.info.bookmarks, id: \.self) { name in
                        Text(name)
                            .jayjayFont(11, design: .monospaced)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(.tint.opacity(0.15), in: .capsule)
                    }
                }
            }
        }
    }

    private var descriptionSection: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text("Description")
                    .jayjayFont(17, weight: .semibold)
                Spacer()
                if editingDescription {
                    Button("Save") {
                        onDescribe(detail.info.changeId, descriptionText)
                        editingDescription = false
                    }
                    .keyboardShortcut("s")
                    Button("Cancel") {
                        descriptionText = detail.info.description
                        editingDescription = false
                    }
                    .keyboardShortcut(.cancelAction)
                } else {
                    Button("Edit") {
                        editingDescription = true
                    }
                }
            }

            if editingDescription {
                TextEditor(text: $descriptionText)
                    .jayjayFont(13, design: .monospaced)
                    .frame(minHeight: 80)
                    .border(.separator)
            } else if detail.info.description.isEmpty {
                Text("(no description)")
                    .foregroundStyle(.tertiary)
                    .italic()
            } else {
                Text(detail.info.description)
                    .jayjayFont(13, design: .monospaced)
                    .textSelection(.enabled)
            }
        }
    }

    @ViewBuilder
    private func previewSection(for hunk: DiffHunk) -> some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Image(systemName: iconName(for: hunk.hunkType))
                    .foregroundStyle(iconColor(for: hunk.hunkType))
                Text(hunk.path)
                    .jayjayFont(14, weight: .semibold, design: .monospaced)
                    .textSelection(.enabled)
                Spacer()
                Text(label(for: hunk.hunkType))
                    .jayjayFont(11, weight: .semibold)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(iconColor(for: hunk.hunkType).opacity(0.12), in: Capsule())
            }

            if hunk.oldContent == nil && hunk.newContent == nil {
                Text("No textual preview available for this file.")
                    .foregroundStyle(.secondary)
            } else {
                MonacoDiffSection(hunk: hunk)
            }
        }
    }

    private func iconName(for type: HunkType) -> String {
        switch type {
        case .added: "plus.circle.fill"
        case .removed: "minus.circle.fill"
        case .modified: "pencil.circle.fill"
        }
    }

    private func iconColor(for type: HunkType) -> Color {
        switch type {
        case .added: .green
        case .removed: .red
        case .modified: .orange
        }
    }

    private func label(for type: HunkType) -> String {
        switch type {
        case .added: "Added"
        case .removed: "Removed"
        case .modified: "Modified"
        }
    }

    private func formatTimestamp(_ millis: Int64) -> String {
        let date = Date(timeIntervalSince1970: Double(millis) / 1000.0)
        return date.formatted(.dateTime.year().month().day().hour().minute())
    }

    private func resetState() {
        descriptionText = detail.info.description
        editingDescription = false
        selectedPath = detail.diff.first?.path
    }

    private func showInFinder(_ path: String) {
        let url = URL(fileURLWithPath: repoPath).appendingPathComponent(path)
        NSWorkspace.shared.activateFileViewerSelecting([url])
    }
}

private struct FileRow: View {
    let hunk: DiffHunk
    let isSelected: Bool

    var body: some View {
        HStack(spacing: 10) {
            Circle()
                .fill(color)
                .frame(width: 8, height: 8)

            VStack(alignment: .leading, spacing: 4) {
                Text(URL(fileURLWithPath: hunk.path).lastPathComponent)
                    .jayjayFont(13, weight: .medium)
                    .lineLimit(1)
                Text(hunk.path)
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            Spacer()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .background(isSelected ? color.opacity(0.14) : Color.primary.opacity(0.035), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
    }

    private var color: Color {
        switch hunk.hunkType {
        case .added: .green
        case .removed: .red
        case .modified: .orange
        }
    }
}

private struct MonacoDiffSection: View {
    let hunk: DiffHunk

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Diff")
                    .jayjayFont(13, weight: .semibold)
                    .foregroundStyle(.secondary)
                Spacer()
                if hunk.oldContent == nil {
                    Text("New file")
                        .jayjayFont(11, weight: .semibold)
                        .foregroundStyle(.green)
                } else if hunk.newContent == nil {
                    Text("Deleted file")
                        .jayjayFont(11, weight: .semibold)
                        .foregroundStyle(.red)
                }
            }

            MonacoDiffView(
                path: hunk.path,
                original: hunk.oldContent ?? "",
                modified: hunk.newContent ?? ""
            )
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 16, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .stroke(Color.primary.opacity(0.08), lineWidth: 1)
            )
        }
    }
}

struct LabeledRow: View {
    let label: String
    let value: String

    init(_ label: String, value: String) {
        self.label = label
        self.value = value
    }

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(label)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .frame(width: 70, alignment: .trailing)
            Text(value)
                .jayjayFont(11, design: .monospaced)
                .textSelection(.enabled)
        }
    }
}
