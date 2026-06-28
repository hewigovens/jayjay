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
