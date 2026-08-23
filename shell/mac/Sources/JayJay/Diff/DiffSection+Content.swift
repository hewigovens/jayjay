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
            .diffCardChrome()
        } else if isSvgFile, activeSvgRichView {
            SvgDiffView(
                oldContent: loadedOldContent ?? hunk.oldContent,
                newContent: loadedNewContent ?? hunk.newContent,
                hunkType: hunk.hunkType
            )
            .diffCardChrome()
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
        } else if shouldShowBlockingProgress {
            ProgressView()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if canRenderMarkdownPreview {
            diffCardWithGutter {
                MarkdownDiffView(
                    markdown: loadedNewContent ?? hunk.newContent,
                    location: markdownPreviewLocation
                )
            }
        } else if canRenderHTMLPreview, let htmlPreviewLocation {
            diffCardWithGutter {
                HTMLDiffView(location: htmlPreviewLocation)
            }
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
                            if reviewNotesEnabled, !loadedReviewNoteSummaries().isEmpty {
                                sideBySideNotesBanner
                                Divider()
                            }
                            SideBySideDiffView(
                                diff: diff,
                                onExpandContext: expandContext,
                                resetSelectionGeneration: contextExpansion.selectionResetGeneration,
                                revealFeedback: contextExpansion.revealFeedback
                            )
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
                            compactGutterWidth: usesProjectionNativeGutter,
                            onExpandContext: expandContext,
                            resetSelectionGeneration: contextExpansion.selectionResetGeneration,
                            reviewStateGeneration: reviewStore?.marksVersion ?? 0,
                            revealFeedback: contextExpansion.revealFeedback
                        )
                        .id("unified-\(hunk.path)")
                    }
                }
                .diffCardChrome()
            }
        } else if hunk.oldContent == nil, hunk.newContent == nil, !isComputing, loadedPath == hunk.path {
            Text("No textual preview available for this file.")
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
        }
    }

    private func diffCardWithGutter(@ViewBuilder content: () -> some View) -> some View {
        HStack(spacing: 0) {
            Color.clear
                .frame(width: richPreviewGutterWidth)
            Rectangle()
                .fill(Color.primary.opacity(0.08))
                .frame(width: 1)
            content()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .diffCardChrome()
    }

    private var richPreviewGutterWidth: CGFloat {
        let font = settings.fontFamily.nsFont(size: CGFloat(settings.fontSize))
        return Self.richPreviewGutterWidth(
            font: font,
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

    var usesProjectionNativeGutter: Bool {
        Self.usesProjectionNativeGutter(projection: effectiveProjection)
    }

    nonisolated static func usesProjectionNativeGutter(projection: DiffProjection?) -> Bool {
        projection != nil
    }

    /// Rooted at the repo checkout, not the file's directory, so parent-relative asset references stay resolvable and contained; the failable init rejects hunk paths that escape the root.
    private var markdownPreviewLocation: RepoPreviewLocation? {
        guard canRenderMarkdownFilePreview, let repoPath = repo?.path() else { return nil }
        return RepoPreviewLocation(root: URL(fileURLWithPath: repoPath, isDirectory: true), relativePath: hunk.path)
    }

    /// Loads the real working-copy file through the scheme handler, not diff content; canOpenHTMLExternally already verified the file exists inside the repo.
    private var htmlPreviewLocation: RepoPreviewLocation? {
        guard canOpenHTMLExternally, let repoPath = repo?.path() else { return nil }
        return RepoPreviewLocation(root: URL(fileURLWithPath: repoPath, isDirectory: true), relativePath: hunk.path)
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
        .diffCardChrome()
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

    private func projectionBanner(_ projection: DiffProjection) -> some View {
        HStack(spacing: 8) {
            Image(systemName: projection.mode == .raw ? "doc.text" : projection.renderKind.iconName)
                .foregroundStyle(projection.diagnostics.isEmpty ? Color.accentColor : .orange)
            Text(DiffProjectionDisplayPolicy.title(for: projection))
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

private extension View {
    func diffCardChrome() -> some View {
        frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(Color.primary.opacity(0.08), lineWidth: 1)
            )
    }
}
