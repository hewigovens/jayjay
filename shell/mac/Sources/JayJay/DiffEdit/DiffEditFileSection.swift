import JayJayCore
import SwiftUI

struct DiffEditFileSection: View {
    let hunk: DiffHunk
    let rev: String
    let repo: JayJayRepo?
    let selectedChangedLines: Set<Int>
    let onToggleFile: () -> Void
    let onSelectFile: () -> Void
    let onToggleLine: (Int) -> Void
    let onSelectHunk: (ClosedRange<Int>) -> Void
    let onLoaded: (DiffEditLoadedFile) -> Void

    @State private var fileDiff: FileDiff?
    @State private var oldContent: String?
    @State private var newContent: String?
    @State private var loadError: String?
    @State private var isLoading = false

    @Environment(AppSettings.self) private var settings

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            header
            content
        }
        .padding(14)
        .background(Color.primary.opacity(0.025), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(
                    selectedChangedLines.isEmpty ? Color.primary.opacity(0.08) : Color.accentColor.opacity(0.35),
                    lineWidth: 1
                )
        )
        .task(id: "\(rev)|\(hunk.path)|\(settings.ignoreWhitespace)") {
            await loadDiff()
        }
    }

    private var header: some View {
        HStack(spacing: 8) {
            if supportsDiffEdit {
                Button(action: onToggleFile) {
                    Text(fileCheckboxText)
                        .jayjayFont(12, weight: .semibold, design: .monospaced)
                        .foregroundStyle(selectedChangedLines.isEmpty ? .secondary : Color.accentColor)
                }
                .buttonStyle(.plain)
            }
            Image(systemName: iconName(for: hunk.hunkType))
                .foregroundStyle(iconColor(for: hunk.hunkType))
            Text(hunk.path)
                .jayjayFont(13, weight: .semibold, design: .monospaced)
                .textSelection(.enabled)
            if supportsDiffEdit, let fileDiff {
                Text(selectionBadgeText(fileDiff: fileDiff))
                    .jayjayFont(10, weight: .semibold)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.accentColor.opacity(0.14), in: Capsule())
            }
            Spacer()
            if supportsDiffEdit {
                Text("Select files or lines to edit")
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            } else {
                Text("Text edits not supported")
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
        }
    }

    @ViewBuilder
    private var content: some View {
        if isLoading {
            ProgressView()
                .frame(maxWidth: .infinity, minHeight: 120)
        } else if let loadError {
            Text(loadError)
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        } else if let fileDiff {
            NativeDiffView(
                diff: fileDiff,
                gutterActions: supportsDiffEdit
                    ? DiffGutterContextActions(
                        openDiffEdit: nil,
                        selectFile: onSelectFile,
                        selectHunk: onSelectHunk,
                        lineCheckboxState: { lineNumber in
                            lineCheckboxState(fileDiff: fileDiff, lineNumber: lineNumber)
                        },
                        toggleLineCheckbox: onToggleLine
                    ) : nil
            )
            .frame(height: diffHeight(for: fileDiff))
        } else {
            Text("No textual preview available for this file.")
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, minHeight: 120)
        }
    }

    private var supportsDiffEdit: Bool {
        hunk.hunkType != .renamed
            && DiffPlaceholder.isEditableText(oldContent)
            && DiffPlaceholder.isEditableText(newContent)
    }

    private var fileCheckboxText: String {
        selectedChangedLines.isEmpty ? "[ ]" : "[x]"
    }

    private func loadDiff() async {
        guard let repo else { return }

        isLoading = true
        loadError = nil

        let loaded = await loadFileContent(repo: repo)
        let old = loaded.0
        let new = loaded.1
        let ignoreWhitespace = settings.ignoreWhitespace

        let diff = await Task.detached {
            repo.computeNativeDiffFull(
                path: hunk.path,
                oldContent: old ?? "",
                newContent: new ?? "",
                ignoreWhitespace: ignoreWhitespace
            )
        }.value

        await MainActor.run {
            oldContent = old
            newContent = new
            fileDiff = diff
            isLoading = false
            onLoaded(DiffEditLoadedFile(hunk: hunk, oldContent: old, newContent: new, diff: diff))
        }
    }

    private func loadFileContent(repo: JayJayRepo) async -> (String?, String?) {
        if hunk.hunkType == .renamed, let oldPath = hunk.oldPath {
            let renamedHunk = await Task.detached {
                try? repo.showFileRename(rev: rev, oldPath: oldPath, newPath: hunk.path)
            }.value
            return (renamedHunk?.oldContent, renamedHunk?.newContent)
        }

        let loaded = await Task.detached {
            try? repo.showFile(rev: rev, path: hunk.path)
        }.value
        if let loaded {
            let old = loaded.oldContent
            let new = loaded.newContent
            if old != nil || new != nil {
                return (old, new)
            }
        }

        let content = await Task.detached {
            try? repo.fileContent(rev: rev, path: hunk.path)
        }.value
        return (nil, content)
    }

    private func diffHeight(for diff: FileDiff) -> CGFloat {
        let lineHeight = max(18, CGFloat(settings.fontSize) + 5)
        return min(max(CGFloat(max(diff.lines.count, 4)) * lineHeight + 24, 120), 680)
    }

    private func iconName(for type: HunkType) -> String {
        switch type {
            case .added: "plus.circle.fill"
            case .removed: "minus.circle.fill"
            case .modified: "pencil.circle.fill"
            case .renamed: "arrow.right.circle.fill"
        }
    }

    private func iconColor(for type: HunkType) -> Color {
        switch type {
            case .added: .green
            case .removed: .red
            case .modified: .orange
            case .renamed: .blue
        }
    }

    private func selectionBadgeText(fileDiff: FileDiff) -> String {
        let changedLineCount = fileDiff.lines.filter(\.isChanged).count
        let selectedLineCount = fileDiff.lines.enumerated().reduce(into: 0) { count, entry in
            let lineNumber = entry.offset + 1
            if entry.element.isChanged, selectedChangedLines.contains(lineNumber) {
                count += 1
            }
        }
        if selectedLineCount == changedLineCount {
            return "File"
        }
        if selectedLineCount == 0 {
            return "None"
        }
        return "\(selectedLineCount) / \(changedLineCount) lines"
    }

    private func lineCheckboxState(fileDiff: FileDiff, lineNumber: Int) -> DiffGutterCheckboxState? {
        let lineIndex = lineNumber - 1
        guard fileDiff.lines.indices.contains(lineIndex) else { return nil }
        guard fileDiff.lines[lineIndex].isChanged else { return nil }
        return selectedChangedLines.contains(lineNumber) ? .selected : .unselected
    }
}

private extension DiffLine {
    var isChanged: Bool {
        style == .added || style == .removed
    }
}
