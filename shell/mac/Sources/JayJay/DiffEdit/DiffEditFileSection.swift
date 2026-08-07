import JayJayCore
import JayJayDiffUI
import SwiftUI

struct DiffEditFileSection: View, DiffGutterSelectionActions {
    let hunk: DiffHunk
    let rev: String
    var commitId: String?
    let repo: JayJayRepo?
    let diffStore: DiffStore
    let selection: DiffEditFileSelectionState
    let stats: FileDiffStats?
    let isCollapsed: Bool
    let isFocused: Bool
    let onToggleCollapse: () -> Void
    let onToggleFile: () -> Void
    let onSelectFile: () -> Void
    let onToggleLine: (Int) -> Void
    let onSelectHunk: (ClosedRange<Int>) -> Void
    let onLoaded: (DiffEditLoadedFile) -> Void

    @State var fileDiff: FileDiff?
    /// Collapsed version for display, with index map back to full diff.
    @State private var displayDiff: FileDiff?
    @State var displayToFullMap: [Int: Int] = [:]
    @State private var oldContent: String?
    @State private var newContent: String?
    @State private var loadError: String?
    @State private var isLoading = false
    @State private var measuredHeight: CGFloat?
    @State private var loadedKey: String?
    @State private var contentGeneration: UInt64 = 0
    @State private var changedLineCount = 0

    private var loadKey: String {
        "\(rev)|\(hunk.path)|\(settings.ignoreWhitespace)"
    }

    @Environment(AppSettings.self) private var settings
    @Environment(\.jayjayFontSize) private var jayjayFontSize
    @Environment(\.jayjayFontFamily) private var jayjayFontFamily

    var selectedChangedLines: Set<Int> {
        selection.selectedChangedLines
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            header
            if !isCollapsed {
                content
            }
        }
        .padding(14)
        .background(Color.primary.opacity(0.025), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(borderColor, lineWidth: isFocused ? 2 : 1)
        )
        .task(id: loadKey) {
            await loadDiff()
        }
        .environment(\.diffFontSize, jayjayFontSize)
        .environment(\.diffFontFamily, jayjayFontFamily.nsFontName)
    }

    private var header: some View {
        let selection = headerSelection
        return HStack(spacing: 8) {
            Button(action: onToggleCollapse) {
                Image(systemName: isCollapsed ? "chevron.right" : "chevron.down")
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                    .frame(width: 12)
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier(AID.DiffEdit.fileToggle(hunk.path))
            .accessibilityValue(isCollapsed ? "collapsed" : "expanded")
            if supportsDiffEdit {
                Button(action: onToggleFile) {
                    Image(systemName: selection.state.systemImage)
                        .foregroundStyle(
                            selection.state == .none ? Color.secondary.opacity(0.4) : Color.accentColor
                        )
                        .jayjayFont(14)
                }
                .buttonStyle(.plain)
            }
            Image(systemName: hunk.hunkType.iconName)
                .foregroundStyle(hunk.hunkType.iconColor)
            Text(hunk.path)
                .jayjayFont(13, weight: .semibold, design: .monospaced)
                .textSelection(.enabled)
            if supportsDiffEdit, let partialText = selection.partialText {
                Text(partialText)
                    .jayjayFont(10, weight: .semibold)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.accentColor.opacity(0.14), in: Capsule())
            }
            statsLabel
            // Toggle lives on the filler, not the whole header, so the path's text selection and checkbox keep their own taps.
            HStack(spacing: 8) {
                Spacer(minLength: 12)
                hintText
            }
            .contentShape(Rectangle())
            .onTapGesture(perform: onToggleCollapse)
        }
    }

    @ViewBuilder
    private var hintText: some View {
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

    @ViewBuilder
    private var statsLabel: some View {
        if let stats, stats.insertions > 0 || stats.deletions > 0 {
            HStack(spacing: 4) {
                if stats.insertions > 0 {
                    Text("+\(stats.insertions)")
                        .jayjayFont(11, weight: .semibold, design: .monospaced)
                        .foregroundStyle(.green)
                }
                if stats.deletions > 0 {
                    Text("-\(stats.deletions)")
                        .jayjayFont(11, weight: .semibold, design: .monospaced)
                        .foregroundStyle(.red)
                }
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
                    : nil,
                contentGeneration: contentGeneration,
                onContentHeightChanged: { height in
                    if abs((measuredHeight ?? 0) - height) > 0.5 {
                        measuredHeight = height
                    }
                }
            )
            .frame(height: measuredHeight ?? estimatedHeight(for: displayDiff))
        } else {
            Text("No textual preview available for this file.")
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, minHeight: 120)
        }
    }

    private var supportsDiffEdit: Bool {
        hunk.projection == nil
            && hunk.hunkType != .renamed
            && DiffPlaceholder.isEditableText(oldContent)
            && DiffPlaceholder.isEditableText(newContent)
    }

    enum FileSelectionState {
        case none, partial, all

        var systemImage: String {
            switch self {
                case .none: "circle"
                case .partial: "minus.circle.fill"
                case .all: "checkmark.circle.fill"
            }
        }
    }

    private var headerSelection: (state: FileSelectionState, partialText: String?) {
        let changed = changedLineCount
        let selected = selectedChangedLines.count
        if selected == 0 || changed == 0 {
            return (.none, nil)
        }
        if selected == changed {
            return (.all, nil)
        }
        return (.partial, "\(selected) / \(changed) lines")
    }

    private var borderColor: Color {
        if isFocused {
            return Color.accentColor.opacity(0.7)
        }
        return selectedChangedLines.isEmpty ? Color.primary.opacity(0.08) : Color.accentColor.opacity(0.35)
    }

    private func loadDiff() async {
        guard let repo else { return }
        // Skip only when this exact rev+mode is loaded (LazyVStack re-triggers .task on scroll); a whitespace-mode change must reload so rows, badges, and apply agree.
        let key = loadKey
        if loadedKey == key {
            return
        }

        isLoading = true
        loadError = nil

        // Reuse DiffStore for file content loading (cached if already loaded by DiffSection); the request mode keeps auto-open formats on the processed rows their stats count.
        let cached = await diffStore.loadDiff(
            hunk: hunk, rev: rev, commitId: commitId, repo: repo,
            ignoreWhitespace: settings.ignoreWhitespace,
            projectionMode: DiffProjectionDisplayPolicy.requestMode(for: hunk.projection, richView: false)
        )
        let loaded = await DiffEditLoadedFile.make(
            hunk: hunk, oldContent: cached?.content.oldContent, newContent: cached?.content.newContent,
            repo: repo, ignoreWhitespace: settings.ignoreWhitespace
        )
        // The detached work outlives .task(id:) cancellation; a superseded mode's result must not install over the replacement's.
        guard !Task.isCancelled, loadKey == key else { return }

        oldContent = loaded.oldContent
        newContent = loaded.newContent
        fileDiff = loaded.diff
        contentGeneration &+= 1
        changedLineCount = loaded.changedLineSet.count

        // Collapse context for display, with mapping back to full diff line numbers
        let collapsed = repo.collapseDiffWithMapping(diff: loaded.diff)
        displayDiff = collapsed.diff
        displayToFullMap = Dictionary(
            uniqueKeysWithValues: collapsed.displayToFull.map {
                (Int($0.displayLine), Int($0.fullLine))
            }
        )

        isLoading = false
        loadedKey = key
        onLoaded(loaded)
    }

    /// Placeholder until the first real layout reports; full content height, never an inner-scroll cap.
    private func estimatedHeight(for diff: FileDiff) -> CGFloat {
        let lineHeight = max(18, CGFloat(settings.fontSize) + 5)
        return max(CGFloat(max(diff.lines.count, 1)) * lineHeight + 24, 44)
    }
}
