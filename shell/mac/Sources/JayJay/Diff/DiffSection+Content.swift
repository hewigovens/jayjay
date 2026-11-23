import AppKit
import Foundation
import JayJayCore
import JayJayDiffUI
import SwiftUI

extension DiffSection {
    private var effectiveOldPreview: DiffPreview? {
        loadedOldPreview ?? hunk.oldPreview
    }

    private var effectiveNewPreview: DiffPreview? {
        loadedNewPreview ?? hunk.newPreview
    }

    private var isImageDiff: Bool {
        effectiveOldPreview?.imagePath != nil || effectiveNewPreview?.imagePath != nil
    }

    @ViewBuilder
    var diffContent: some View {
        if isImageDiff {
            ImageDiffView(
                oldPath: effectiveOldPreview?.imagePath,
                newPath: effectiveNewPreview?.imagePath,
                hunkType: hunk.hunkType
            )
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if isSvgFile, activeSvgRichView {
            SvgDiffView(
                oldContent: loadedOldContent ?? hunk.oldContent,
                newContent: loadedNewContent ?? hunk.newContent,
                hunkType: hunk.hunkType
            )
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let placeholder {
            placeholder
        } else if let loadError {
            DiffLoadErrorView(message: loadError) { loadAttempt += 1 }
        } else if shouldShowBlockingProgress {
            LoadingStatus(label: Localized.Diff.loadingDiffEllipsis)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if canRenderMarkdownPreview {
            richPreviewWithGutter {
                MarkdownDiffView(
                    markdown: loadedNewContent ?? hunk.newContent,
                    location: markdownPreviewLocation
                )
            }
        } else if canRenderHTMLPreview, let htmlPreviewLocation {
            richPreviewWithGutter {
                HTMLDiffView(location: htmlPreviewLocation)
            }
        } else if foldsLargeDiff, let onShowAlone {
            HStack(spacing: 8) {
                Text(Localized.Diff.largeDiffFolded(count: fileDiff?.lines.count ?? 0))
                    .foregroundStyle(.secondary)
                Button(Localized.Diff.showDiff, action: onShowAlone)
                    .buttonStyle(InlineTextButtonStyle())
                    .accessibilityIdentifier(AID.Diff.showLargeDiff)
            }
            .jayjayFont(12)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if hasCurrentRenderableDiff, let diff = fileDiff, !diff.lines.isEmpty {
            VStack(alignment: .leading, spacing: 6) {
                if diff.whitespaceOnlyHidden {
                    whitespaceHiddenBanner
                }
                if let projection = effectiveProjection, shouldShowProjectionBanner {
                    projectionBanner(projection)
                }
                Group {
                    if settings.sideBySideDiff, canUseSideBySide(diff) {
                        VStack(alignment: .leading, spacing: 0) {
                            // Side-by-side has no review gutter yet; without this bridge the file's advertised notes would be unreachable until the user guesses to switch views.
                            if reviewNotesEnabled, !unresolvedReviewNoteSummaries.isEmpty {
                                sideBySideNotesBanner
                                Divider()
                            }
                            SideBySideDiffView(
                                diff: diff,
                                onExpandContext: expandContext,
                                resetSelectionGeneration: contextExpansionDisplay.selectionGeneration,
                                revealFeedback: contextExpansionDisplay.revealFeedback,
                                onContentHeightChanged: fitsContent ? { fittedDiffHeight = $0 } : nil
                            )
                            .frame(height: fittedHeight(of: diff))
                        }
                        .id("sbs-\(hunk.path)")
                    } else {
                        NativeDiffView(
                            diff: diff,
                            gutterActions: self,
                            reviewNotes: reviewNotesEnabled ? loadedReviewNoteSummaries() : [],
                            displayLines: loadedDisplayLines,
                            displayGroups: displayGroups,
                            reserveNoteColumn: reservesReviewNoteGutterColumn,
                            compactGutterWidth: effectiveProjection != nil,
                            onExpandContext: expandContext,
                            resetSelectionGeneration: contextExpansionDisplay.selectionGeneration,
                            reviewStateGeneration: reviewStore?.marksVersion ?? 0,
                            revealFeedback: contextExpansionDisplay.revealFeedback,
                            onContentHeightChanged: fitsContent ? { fittedDiffHeight = $0 } : nil
                        )
                        .frame(height: fittedHeight(of: diff))
                        .id("unified-\(hunk.path)")
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: fittedDiffHeight == nil ? .infinity : nil)
            }
        } else if hunk.oldContent == nil, hunk.newContent == nil, !isComputing, loadedPath == hunk.path {
            Text(Localized.Repo.noTextualPreviewAvailableForThis)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
        }
    }

    private func fittedHeight(of diff: FileDiff) -> CGFloat? {
        fitsContent ? fittedDiffHeight ?? diff.estimatedCardHeight(typography: settings.typography) : nil
    }

    private func richPreviewWithGutter(@ViewBuilder content: () -> some View) -> some View {
        HStack(spacing: 0) {
            Color.clear
                .frame(width: richPreviewGutterWidth)
            Rectangle()
                .fill(Color.primary.opacity(0.08))
                .frame(width: 1)
            content()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    private var richPreviewGutterWidth: CGFloat {
        Self.richPreviewGutterWidth(
            font: settings.typography.editorFont,
            showsNoteColumn: reservesReviewNoteGutterColumn,
            hasVisibleNoteMarker: !loadedReviewNoteSummaries().isEmpty
        )
    }

    nonisolated static func richPreviewGutterWidth(
        font: NSFont,
        showsNoteColumn: Bool,
        hasVisibleNoteMarker: Bool
    ) -> CGFloat {
        DiffGutterMetrics.richPreviewWidth(
            font: font,
            showsNoteColumn: showsNoteColumn,
            hasVisibleNoteMarker: hasVisibleNoteMarker
        )
    }

    /// Rooted at the repo checkout, not the file's directory, so parent-relative asset references stay resolvable and contained; the failable init rejects hunk paths that escape the root.
    private var markdownPreviewLocation: RepoPreviewLocation? {
        guard canRenderMarkdownFilePreview, let repoPath = repo?.path(), !repoPath.isSSHLocation else { return nil }
        return RepoPreviewLocation(root: URL(fileURLWithPath: repoPath, isDirectory: true), relativePath: hunk.path)
    }

    /// Loads the real working-copy file through the scheme handler, not diff content; canOpenHTMLExternally already verified the file exists inside the repo.
    private var htmlPreviewLocation: RepoPreviewLocation? {
        guard canOpenHTMLExternally, let repoPath = repo?.path(), !repoPath.isSSHLocation else { return nil }
        return RepoPreviewLocation(root: URL(fileURLWithPath: repoPath, isDirectory: true), relativePath: hunk.path)
    }

    private var placeholder: DiffPlaceholderMessage? {
        guard !isImageDiff, !(isSvgFile && activeSvgRichView) else { return nil }
        if hunk.isSubmodulePlaceholder {
            return DiffPlaceholderMessage(systemImage: "shippingbox.fill", title: Localized.Diff.gitSubmodule, description: Localized.Diff.submoduleShowsCommitOnly)
        }
        if isGitLfsPlaceholder {
            return DiffPlaceholderMessage(
                systemImage: "externaldrive.fill.badge.timemachine",
                title: Localized.Diff.gitLfsBackedFile,
                description: Localized.Diff.thisFileIsTrackedThroughGit
            )
        }
        guard hunk.isContentFreeRenameOrCopy else { return nil }
        let isCopy = hunk.hunkType == .copied
        return DiffPlaceholderMessage(
            systemImage: isCopy ? "square.on.square" : "arrow.right.circle",
            title: Localized.Diff.noContentChanges,
            description: isCopy ? Localized.Diff.thisFileWasCopiedItsContents : Localized.Diff.thisFileWasRenamedItsContents
        )
    }

    private var whitespaceHiddenBanner: some View {
        HStack(spacing: 8) {
            Image(systemName: "eye.slash")
                .foregroundStyle(.orange)
            Text(Localized.Diff.whitespaceOnlyChangesHiddenByThe)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Spacer()
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(Color.orange.opacity(0.08), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
        .padding(.horizontal, PaneLayout.detailInset)
    }

    private func projectionBanner(_ projection: DiffProjection) -> some View {
        HStack(spacing: 8) {
            Image(systemName: projection.mode == .raw ? "doc.text" : projection.renderKind.iconName)
                .foregroundStyle(projection.diagnostics.isEmpty ? Color.accentColor : .orange)
            Text(projectionTitle(projection: projection))
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .lineLimit(1)
                .truncationMode(.tail)
            Spacer()
            if !projection.diagnostics.isEmpty {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.orange)
                    .help(projection.diagnostics.joined(separator: "\n"))
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(
            (projection.diagnostics.isEmpty ? Color.accentColor : Color.orange).opacity(0.08),
            in: RoundedRectangle(cornerRadius: 8, style: .continuous)
        )
        .padding(.horizontal, PaneLayout.detailInset)
    }

    private var unresolvedReviewNoteSummaries: [DiffReviewNoteSummary] {
        loadedReviewNoteSummaries().filter { !$0.isResolved }
    }

    private var sideBySideNotesBanner: some View {
        let count = unresolvedReviewNoteSummaries.count
        return HStack(spacing: 8) {
            Image(systemName: "text.bubble.fill")
                .foregroundStyle(.orange)
            Text(Localized.Diff.reviewNotesOnFile(count: count))
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Spacer()
            Button(Localized.Diff.showInUnified) {
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

    /// Fitting a diff to its content lays out every line at once, so a larger one opens alone, in its own scroll view.
    private static let stackedLineLimit = 3000

    private var foldsLargeDiff: Bool {
        onShowAlone != nil && hasCurrentRenderableDiff && (fileDiff?.lines.count ?? 0) > Self.stackedLineLimit
    }

    /// Nil once the diff text reports its own height or a placeholder sizes itself; rich previews and unloaded diffs report none.
    var stackedContentHeight: CGFloat? {
        if placeholder != nil {
            return nil
        }
        if showsRichView {
            return 320
        }
        if foldsLargeDiff {
            return 44
        }
        return fittedDiffHeight == nil ? (awaitsDiffText ? stackedHeights?.height(of: hunk.path, in: heightBasis) : nil) ?? 320 : nil
    }

    var heightBasis: StackedDiffHeights.Basis {
        .init(
            commitId: commitId,
            compareFromRev: compareFromRev,
            ignoreWhitespace: settings.ignoreWhitespace,
            typography: settings.typography,
            columns: fileDiff.map(canUseSideBySide) == false ? .alwaysUnified : settings.sideBySideDiff ? .sideBySide : .unified
        )
    }

    private var awaitsDiffText: Bool {
        loadError == nil && (isComputing || loadedPath != hunk.path || fileDiff?.lines.isEmpty == false)
    }

    func canUseSideBySide(_ diff: FileDiff) -> Bool {
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

    private var isGitLfsPlaceholder: Bool {
        let oldContent = hunk.oldContent ?? loadedOldContent
        let newContent = hunk.newContent ?? loadedNewContent
        return DiffPlaceholder.isGitLfs(oldContent) || DiffPlaceholder.isGitLfs(newContent)
    }
}

private struct DiffPlaceholderMessage: View {
    let systemImage: String
    let title: String
    let description: String

    var body: some View {
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
        .padding(.vertical, 24)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
