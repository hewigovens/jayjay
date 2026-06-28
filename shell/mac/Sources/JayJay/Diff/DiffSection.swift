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

    /// Non-private members are read by the DiffSection+Content / +EditActions / +ReviewActions extensions.
    @State var fileDiff: FileDiff?
    // Display lines and change groups computed once per loaded diff; updateNSView re-runs per observed change and the FFI is O(diff bytes).
    @State var displayGroups: [ChangeGroup]?
    @State var loadedDisplayLines: [DiffLine]?
    @State var isComputing = false
    @State var loadedPath: String?
    @State var loadedOldContent: String?
    @State var loadedNewContent: String?
    @State var loadedOldPreview: DiffPreview?
    @State var loadedNewPreview: DiffPreview?
    @State var selectedLineRange: ClosedRange<Int>?
    @State private var copiedPath = false
    @State var svgRichView = false
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

    var isSvgFile: Bool {
        hunk.path.lowercased().hasSuffix(".svg")
    }

    private var effectiveSideBySideDiff: Bool {
        guard settings.sideBySideDiff else { return false }
        guard let fileDiff else { return true }
        return canUseSideBySide(fileDiff)
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
