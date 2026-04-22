import JayJayCore
import JayJayDiffUI
import SwiftUI

struct DiffSection: View {
    let hunk: DiffHunk
    let rev: String?
    let repo: JayJayRepo?
    let actions: (any ChangeActions & DAGActions)?
    let isWorkingCopy: Bool
    let diffStore: DiffStore
    var onOpenDiffEdit: (() -> Void)?
    var compareFromRev: String?

    @State private var fileDiff: FileDiff?
    @State private var isComputing = false
    @State private var loadedPath: String?
    @State private var loadedOldContent: String?
    @State private var loadedNewContent: String?
    @State private var loadedOldPreview: DiffPreview?
    @State private var loadedNewPreview: DiffPreview?
    @State private var selectedLineRange: ClosedRange<Int>?
    @State private var copiedPath = false
    @State private var svgRichView = false
    @Environment(AppSettings.self) private var settings
    @Environment(\.jayjayFontSize) private var jayjayFontSize
    @Environment(\.jayjayFontFamily) private var jayjayFontFamily

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            diffHeader
            diffContent
        }
        .accessibilityIdentifier(AID.Diff.section)
        .task(id: "\(compareFromRev ?? "")|\(rev ?? "")|\(hunk.path)|\(settings.ignoreWhitespace)") {
            await computeDiffAsync()
        }
        .environment(\.diffFontSize, jayjayFontSize)
        .environment(\.diffFontFamily, jayjayFontFamily.nsFontName)
    }

    private var diffHeader: some View {
        HStack {
            Image(systemName: iconName(for: hunk.hunkType))
                .foregroundStyle(iconColor(for: hunk.hunkType))
            Text(hunk.path)
                .jayjayFont(14, weight: .semibold, design: .monospaced)
                .lineLimit(1)
                .truncationMode(.middle)
                .textSelection(.enabled)
                .help(hunk.path)
            Button {
                copyPath()
            } label: {
                Image(systemName: copiedPath ? "checkmark" : "doc.on.doc")
                    .jayjayFont(11)
                    .foregroundStyle(copiedPath ? Color.green : .secondary)
            }
            .buttonStyle(.plain)
            .help(copiedPath ? "Copied path" : "Copy path")
            if isSvgFile {
                Button {
                    svgRichView.toggle()
                } label: {
                    Image(systemName: svgRichView ? "eye.fill" : "eye")
                        .jayjayFont(11)
                        .foregroundStyle(svgRichView ? Color.accentColor : .secondary)
                }
                .buttonStyle(.plain)
                .help(svgRichView ? "Show text diff" : "Show rendered SVG")
            }
            Spacer()
            if hunk.hunkType == .renamed, let oldPath = hunk.oldPath {
                Text(oldPath)
                    .jayjayFont(11, design: .monospaced)
                    .strikethrough()
                    .foregroundStyle(.secondary)
                Image(systemName: "arrow.right")
                    .jayjayFont(10)
                    .foregroundStyle(.secondary)
            }
            Text(label(for: hunk.hunkType))
                .jayjayFont(11, weight: .semibold)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(iconColor(for: hunk.hunkType).opacity(0.12), in: Capsule())
        }
    }

    private var effectiveOldPreview: DiffPreview? {
        loadedOldPreview ?? hunk.oldPreview
    }

    private var effectiveNewPreview: DiffPreview? {
        loadedNewPreview ?? hunk.newPreview
    }

    private var isImageDiff: Bool {
        effectiveOldPreview?.imagePath != nil || effectiveNewPreview?.imagePath != nil
    }

    private var isSvgFile: Bool {
        hunk.path.lowercased().hasSuffix(".svg")
    }

    @ViewBuilder
    private var diffContent: some View {
        if isImageDiff {
            ImageDiffView(
                oldPath: effectiveOldPreview?.imagePath,
                newPath: effectiveNewPreview?.imagePath,
                hunkType: hunk.hunkType
            )
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(Color.primary.opacity(0.08), lineWidth: 1)
            )
        } else if isSvgFile, svgRichView {
            SvgDiffView(
                oldContent: loadedOldContent ?? hunk.oldContent,
                newContent: loadedNewContent ?? hunk.newContent,
                hunkType: hunk.hunkType
            )
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(Color.primary.opacity(0.08), lineWidth: 1)
            )
        } else if hunk.isSubmodulePlaceholder {
            VStack(spacing: 10) {
                Image(systemName: "shippingbox.fill")
                    .jayjayFont(24)
                    .foregroundStyle(.secondary)
                Text("Git submodule")
                    .jayjayFont(14, weight: .semibold)
                Text(
                    "This submodule has working-copy changes, but JayJay does not render an inline text diff for submodule contents. Open or commit the submodule in its own repository."
                )
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 420)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(Color.primary.opacity(0.08), lineWidth: 1)
            )
        } else if isGitLfsPlaceholder {
            VStack(spacing: 10) {
                Image(systemName: "externaldrive.fill.badge.timemachine")
                    .jayjayFont(24)
                    .foregroundStyle(.secondary)
                Text("Git LFS-backed file")
                    .jayjayFont(14, weight: .semibold)
                Text(
                    "This file is tracked through Git LFS. JayJay does not render an inline text diff between the committed pointer and the local binary object."
                )
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 460)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(Color.primary.opacity(0.08), lineWidth: 1)
            )
        } else if isComputing {
            ProgressView()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let diff = fileDiff, !diff.lines.isEmpty {
            VStack(alignment: .leading, spacing: 6) {
                if diff.whitespaceOnlyHidden {
                    whitespaceHiddenBanner
                }
                Group {
                    if settings.sideBySideDiff, isTwoColumnDiff(diff) {
                        SideBySideDiffView(diff: diff)
                            .id("sbs-\(hunk.path)")
                    } else {
                        NativeDiffView(
                            diff: diff,
                            gutterActions: DiffGutterContextActions(
                                openDiffEdit: onOpenDiffEdit,
                                onLineSelectionChanged: { selectedLineRange = $0 },
                                selectedLineRange: selectedLineRange,
                                abandonSelectedLines: isWorkingCopy ? abandonSelectedLines : nil
                            )
                        )
                        .id("unified-\(hunk.path)")
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .stroke(Color.primary.opacity(0.08), lineWidth: 1)
                )
            }
        } else if hunk.oldContent == nil, hunk.newContent == nil, !isComputing, loadedPath == hunk.path {
            Text("No textual preview available for this file.")
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
        }
    }

    private var whitespaceHiddenBanner: some View {
        HStack(spacing: 8) {
            Image(systemName: "eye.slash")
                .foregroundStyle(.orange)
            Text("Whitespace-only changes hidden by the 'Ignore whitespace' setting.")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Spacer()
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(Color.orange.opacity(0.08), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
    }

    private func isTwoColumnDiff(_ diff: FileDiff) -> Bool {
        let hasAdded = diff.lines.contains { $0.style == .added }
        let hasRemoved = diff.lines.contains { $0.style == .removed }
        return hasAdded && hasRemoved
    }

    private func computeDiffAsync() async {
        guard !hunk.isSubmodulePlaceholder else {
            fileDiff = nil
            loadedOldContent = hunk.oldContent
            loadedNewContent = hunk.newContent
            loadedPath = hunk.path
            isComputing = false
            return
        }

        let path = hunk.path
        isComputing = true
        fileDiff = nil

        if let cached = await diffStore.loadDiff(
            hunk: hunk, rev: rev, repo: repo,
            compareFromRev: compareFromRev,
            ignoreWhitespace: settings.ignoreWhitespace
        ) {
            guard hunk.path == path else { return }
            fileDiff = cached.diff
            loadedOldContent = cached.oldContent
            loadedNewContent = cached.newContent
            loadedOldPreview = cached.oldPreview
            loadedNewPreview = cached.newPreview
            loadedPath = path
        }
        isComputing = false
    }

    // MARK: - Helpers

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

    private func label(for type: HunkType) -> String {
        switch type {
            case .added: "Added"
            case .removed: "Removed"
            case .modified: "Modified"
            case .renamed: "Renamed"
        }
    }

    private var isGitLfsPlaceholder: Bool {
        let oldContent = hunk.oldContent ?? loadedOldContent
        let newContent = hunk.newContent ?? loadedNewContent
        return DiffPlaceholder.isGitLfs(oldContent) || DiffPlaceholder.isGitLfs(newContent)
    }

    private func copyPath() {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(hunk.path, forType: .string)
        copiedPath = true
        Task {
            try? await Task.sleep(for: .seconds(1.5))
            copiedPath = false
        }
    }

    private func abandonSelectedLines() {
        guard let actions,
              let repo,
              let rev,
              let fileDiff,
              let selectedLineRange
        else { return }

        let oldContent = loadedOldContent ?? hunk.oldContent
        let newContent = loadedNewContent ?? hunk.newContent
        let selectedKeys: Set<String> = Set(
            fileDiff.lines.enumerated().compactMap { index, line in
                let displayLine = index + 1
                guard selectedLineRange.contains(displayLine),
                      line.style == .added || line.style == .removed
                else { return nil }
                return diffLineKey(line)
            }
        )
        guard !selectedKeys.isEmpty else { return }

        let fullDiff = repo.computeNativeDiffFull(
            path: hunk.path,
            oldContent: oldContent ?? "",
            newContent: newContent ?? "",
            ignoreWhitespace: settings.ignoreWhitespace
        )
        let fullLineIndices = fullDiff.lines.enumerated().compactMap { index, line in
            selectedKeys.contains(diffLineKey(line)) ? index + 1 : nil
        }
        let ranges = collapsedRanges(fullLineIndices)
        guard !ranges.isEmpty else { return }

        actions.applyDiffSelection(
            rev: rev,
            destination: .removeFromSource,
            selections: [
                DiffEditFileSelection(
                    path: hunk.path,
                    oldPath: hunk.oldPath,
                    oldContent: oldContent,
                    newContent: newContent,
                    hunkType: hunk.hunkType,
                    lineRanges: ranges
                )
            ],
            message: "",
            ignoreWhitespace: settings.ignoreWhitespace
        )
    }

    private func diffLineKey(_ line: DiffLine) -> String {
        let style = switch line.style {
            case .added: "added"
            case .removed: "removed"
            case .context: "context"
            case .separator: "separator"
            case .unchanged: "unchanged"
        }
        return "\(style)|\(line.oldLineNo.map(String.init) ?? "-")|\(line.newLineNo.map(String.init) ?? "-")"
    }

    private func collapsedRanges(_ indices: [Int]) -> [DiffEditRange] {
        guard let first = indices.first else { return [] }

        var ranges: [DiffEditRange] = []
        var start = first
        var previous = first

        for index in indices.dropFirst() {
            if index == previous + 1 {
                previous = index
                continue
            }
            ranges.append(DiffEditRange(startLine: UInt32(start), endLine: UInt32(previous)))
            start = index
            previous = index
        }

        ranges.append(DiffEditRange(startLine: UInt32(start), endLine: UInt32(previous)))
        return ranges
    }
}
