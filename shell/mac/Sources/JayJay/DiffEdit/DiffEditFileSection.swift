import JayJayCore
import JayJayDiffUI
import SwiftUI

struct DiffEditFileSection: View, DiffGutterSelectionActions {
    let hunk: DiffHunk
    let rev: String
    var commitId: String?
    let repo: JayJayRepo?
    let diffStore: DiffStore
    let selectedChangedLines: Set<Int>
    let onToggleFile: () -> Void
    let onSelectFile: () -> Void
    let onToggleLine: (Int) -> Void
    let onSelectHunk: (ClosedRange<Int>) -> Void
    let onLoaded: (DiffEditLoadedFile) -> Void

    @State private var fileDiff: FileDiff?
    /// Collapsed version for display, with index map back to full diff.
    @State private var displayDiff: FileDiff?
    @State private var displayToFullMap: [Int: Int] = [:]
    @State private var oldContent: String?
    @State private var newContent: String?
    @State private var loadError: String?
    @State private var isLoading = false

    @Environment(AppSettings.self) private var settings
    @Environment(\.jayjayFontSize) private var jayjayFontSize
    @Environment(\.jayjayFontFamily) private var jayjayFontFamily

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
        .environment(\.diffFontSize, jayjayFontSize)
        .environment(\.diffFontFamily, jayjayFontFamily.nsFontName)
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
        } else if let displayDiff {
            NativeDiffView(
                diff: displayDiff,
                gutterActions: supportsDiffEdit
                    ? self
                    : nil
            )
            .frame(height: diffHeight(for: displayDiff))
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
        // Already loaded for this file — skip (LazyVStack re-triggers .task on scroll)
        if fileDiff != nil, displayDiff != nil { return }

        isLoading = true
        loadError = nil

        // Reuse DiffStore for file content loading (cached if already loaded by DiffSection)
        let cached = await diffStore.loadDiff(
            hunk: hunk, rev: rev, commitId: commitId, repo: repo,
            ignoreWhitespace: settings.ignoreWhitespace
        )
        let old = cached?.oldContent
        let new = cached?.newContent
        let ignoreWhitespace = settings.ignoreWhitespace
        let path = hunk.path

        // DiffEdit needs the full (uncollapsed) diff — line selection indices
        // must match the full diff the Rust side computes when applying.
        let diff = await Task.detached {
            repo.computeNativeDiffFull(
                path: path, oldContent: old ?? "", newContent: new ?? "",
                ignoreWhitespace: ignoreWhitespace
            )
        }.value

        oldContent = old
        newContent = new
        fileDiff = diff

        // Collapse context for display, with mapping back to full diff line numbers
        let collapsed = repo.collapseDiffWithMapping(diff: diff)
        displayDiff = collapsed.diff
        displayToFullMap = Dictionary(
            uniqueKeysWithValues: collapsed.displayToFull.map {
                (Int($0.displayLine), Int($0.fullLine))
            }
        )

        isLoading = false
        onLoaded(DiffEditLoadedFile(hunk: hunk, oldContent: old, newContent: new, diff: diff))
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
            case .modified: FileStatusColors.modified
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

    var currentSelectedLineRange: ClosedRange<Int>? {
        nil
    }

    func didSelectLines(_ lineRange: ClosedRange<Int>) {}

    func selectFile() {
        onSelectFile()
    }

    func selectChangeGroup(_ lineRange: ClosedRange<Int>) {
        let mapped = ClosedRange(
            uncheckedBounds: (
                displayToFullMap[lineRange.lowerBound] ?? lineRange.lowerBound,
                displayToFullMap[lineRange.upperBound] ?? lineRange.upperBound
            )
        )
        onSelectHunk(mapped)
    }

    func lineCheckboxState(for lineNumber: Int) -> DiffGutterCheckboxState? {
        guard let fileDiff,
              let fullLine = displayToFullMap[lineNumber]
        else { return nil }
        return lineCheckboxState(fileDiff: fileDiff, lineNumber: fullLine)
    }

    func toggleLineCheckbox(_ lineNumber: Int) {
        guard let fullLine = displayToFullMap[lineNumber] else { return }
        onToggleLine(fullLine)
    }
}

private extension DiffLine {
    var isChanged: Bool {
        style == .added || style == .removed
    }
}
