import JayJayCore
import JayJayDiffUI
import SwiftUI

struct DiffSection: View {
    let hunk: DiffHunk
    let rev: String?
    var commitId: String?
    // Review store key: always the real change id. `rev` is the selection revision, which is a commit id for divergent changes and would hide notes from CLI/core reconciliation (they key by change id).
    var reviewChangeId: String?
    let repo: JayJayRepo?
    let actions: (any ChangeActions & DAGActions)?
    let isWorkingCopy: Bool
    let diffStore: DiffStore
    let reviewStore: ReviewStore?
    /// Stale/orphaned note ids from the async reconciliation report; their bubbles must not expand into the diff since their anchors may be wrong.
    var staleNoteIds: Set<String> = []
    // Owned by ChangeDetailView: this view is rebuilt on every commit-id change (background snapshots included), which would reset a local @State editor and dismiss the sheet mid-typing.
    @Binding var noteEditor: ReviewNoteEditorState?
    var onOpenDiffEdit: (() -> Void)?
    var onReviewStateChanged: (() -> Void)?
    var compareFromRev: String?

    /// Non-private members are read by the DiffSection+EditActions / +ReviewActions extensions.
    @State var fileDiff: FileDiff?
    // Display lines and change groups computed once per loaded diff; updateNSView re-runs per observed change and the FFI is O(diff bytes).
    @State var displayGroups: [ChangeGroup]?
    @State var loadedDisplayLines: [DiffLine]?
    @State private var isComputing = false
    @State private var loadedPath: String?
    @State var loadedOldContent: String?
    @State var loadedNewContent: String?
    @State private var loadedOldPreview: DiffPreview?
    @State private var loadedNewPreview: DiffPreview?
    @State var selectedLineRange: ClosedRange<Int>?
    @State private var copiedPath = false
    @State private var svgRichView = false
    @Environment(AppSettings.self) var settings
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
            Image(systemName: hunk.hunkType.iconName)
                .foregroundStyle(hunk.hunkType.iconColor)
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
            Button {
                settings.sideBySideDiff.toggle()
            } label: {
                HStack(spacing: 5) {
                    Image(
                        systemName: effectiveSideBySideDiff
                            ? "rectangle.split.2x1"
                            : "text.justify"
                    )
                    .jayjayFont(11)
                    Text(effectiveSideBySideDiff ? "Side-by-side" : "Unified")
                        .jayjayFont(11)
                }
                .foregroundStyle(effectiveSideBySideDiff ? Color.accentColor : .secondary)
                .padding(.horizontal, 8)
                .padding(.vertical, 3)
                .background(
                    effectiveSideBySideDiff
                        ? AnyShapeStyle(Color.accentColor.opacity(0.14))
                        : AnyShapeStyle(Color.primary.opacity(0.06)),
                    in: RoundedRectangle(cornerRadius: 4, style: .continuous)
                )
            }
            .buttonStyle(.plain)
            .help(effectiveSideBySideDiff ? "Switch to unified" : "Switch to side-by-side")
            Text(label(for: hunk.hunkType))
                .jayjayFont(11, weight: .semibold)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(hunk.hunkType.iconColor.opacity(0.12), in: Capsule())
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
            placeholderCard(
                systemImage: "shippingbox.fill",
                title: "Git submodule",
                description: "This submodule has working-copy changes, but JayJay does not render an inline text diff for submodule contents. Open or commit the submodule in its own repository."
            )
        } else if isGitLfsPlaceholder {
            placeholderCard(
                systemImage: "externaldrive.fill.badge.timemachine",
                title: "Git LFS-backed file",
                description: "This file is tracked through Git LFS. JayJay does not render an inline text diff between the committed pointer and the local binary object."
            )
        } else if hunk.isContentFreeRename {
            placeholderCard(
                systemImage: "arrow.right.circle",
                title: "No content changes",
                description: "This file was renamed; its contents are identical."
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
                    if settings.sideBySideDiff, canUseSideBySide(diff) {
                        VStack(alignment: .leading, spacing: 0) {
                            // Side-by-side has no review gutter yet; without this bridge the file's advertised notes would be unreachable until the user guesses to switch views.
                            if reviewNotesEnabled, !loadedReviewNoteSummaries().isEmpty {
                                sideBySideNotesBanner
                                Divider()
                            }
                            SideBySideDiffView(diff: diff)
                        }
                        .id("sbs-\(hunk.path)")
                    } else {
                        NativeDiffView(
                            diff: diff,
                            gutterActions: self,
                            reviewNotes: reviewNotesEnabled ? loadedReviewNoteSummaries() : [],
                            displayLines: loadedDisplayLines,
                            displayGroups: displayGroups
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

    private func placeholderCard(systemImage: String, title: String, description: String) -> some View {
        VStack(spacing: 10) {
            Image(systemName: systemImage)
                .jayjayFont(24)
                .foregroundStyle(.secondary)
            Text(title)
                .jayjayFont(14, weight: .semibold)
            Text(description)
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

    private var sideBySideNotesBanner: some View {
        let count = loadedReviewNoteSummaries().count
        return HStack(spacing: 8) {
            Image(systemName: "text.bubble.fill")
                .foregroundStyle(.orange)
            Text(count == 1 ? "1 review note on this file" : "\(count) review notes on this file")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Spacer()
            Button("Show in Unified") {
                settings.sideBySideDiff = false
            }
            .buttonStyle(.plain)
            .jayjayFont(11, weight: .medium)
            .foregroundStyle(Color.accentColor)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(Color.orange.opacity(0.08))
    }

    private var effectiveSideBySideDiff: Bool {
        guard settings.sideBySideDiff else { return false }
        guard let fileDiff else { return true }
        return canUseSideBySide(fileDiff)
    }

    private func canUseSideBySide(_ diff: FileDiff) -> Bool {
        isTwoColumnDiff(diff) && !hasConflictLines(diff)
    }

    private func isTwoColumnDiff(_ diff: FileDiff) -> Bool {
        let hasAdded = diff.lines.contains { $0.style == .added }
        let hasRemoved = diff.lines.contains { $0.style == .removed }
        return hasAdded && hasRemoved
    }

    private func hasConflictLines(_ diff: FileDiff) -> Bool {
        diff.lines.contains { $0.conflictKind != .none }
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

        guard !hunk.isContentFreeRename else {
            fileDiff = nil
            loadedPath = hunk.path
            isComputing = false
            return
        }

        let path = hunk.path
        if let cached = await diffStore.cachedDiff(
            hunk: hunk, rev: rev, commitId: commitId,
            compareFromRev: compareFromRev,
            ignoreWhitespace: settings.ignoreWhitespace
        ) {
            // Bail if a newer .task superseded us; nothing below checks cancellation, so without this a stale diff overwrites fresh @State.
            guard !Task.isCancelled, hunk.path == path else {
                isComputing = false
                return
            }
            apply(cached, path: path)
            isComputing = false
            return
        }

        isComputing = true
        fileDiff = nil

        if let cached = await diffStore.loadDiff(
            hunk: hunk, rev: rev, commitId: commitId, repo: repo,
            compareFromRev: compareFromRev,
            ignoreWhitespace: settings.ignoreWhitespace
        ) {
            // Same supersession bail as the cached path: loadDiff never observes cancellation.
            guard !Task.isCancelled, hunk.path == path else {
                isComputing = false
                return
            }
            apply(cached, path: path)
        }
        isComputing = false
    }

    private func apply(_ cached: DiffStore.CachedDiff, path: String) {
        fileDiff = cached.diff
        let lines = diffDisplayLines(lines: cached.diff.lines)
        loadedDisplayLines = lines
        displayGroups = changeGroups(lines: lines)
        loadedOldContent = cached.oldContent
        loadedNewContent = cached.newContent
        loadedOldPreview = cached.oldPreview
        loadedNewPreview = cached.newPreview
        refreshActiveNotes()
        loadedPath = path
    }

    // MARK: - Helpers

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
}
