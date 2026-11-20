import JayJayCore
import SwiftUI

struct FileRow: View {
    @Environment(\.jayjayTypography) private var typography
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.appThemes) private var appThemes
    /// macOS List adds 8 pt of its own on each side; this nets out to a 4 pt inset so the selection covers the row.
    static let listInsets = EdgeInsets(top: 0, leading: -4, bottom: 0, trailing: -4)

    let hunk: DiffHunk
    let isSelected: Bool
    var isPaneActive = true
    var showReview: Bool = false
    var reviewRollup: ReviewFileRollup = .unreviewed
    var agentMarked: Bool = false
    var noteCount: Int = 0
    var hasConflict: Bool = false
    var onToggleReview: (() -> Void)?

    private var movedFrom: String? {
        guard hunk.hunkType.hasSource, let oldPath = hunk.oldPath, oldPath != hunk.path else { return nil }
        return oldPath
    }

    var pathDetail: String? {
        if let movedFrom {
            return "\(movedFrom) → \(hunk.path)"
        }
        guard let slash = hunk.path.lastIndex(of: "/") else { return nil }
        return String(hunk.path[..<slash])
    }

    var reviewChrome: FileRowReviewChrome {
        FileRowReviewChrome.chrome(showReview: showReview, rollup: reviewRollup)
    }

    var showsReviewedStyle: Bool {
        reviewChrome == .reviewed
    }

    var showsAgentBadge: Bool {
        showReview && agentMarked && reviewChrome != .unreviewed
    }

    var reviewAccessibilityLabel: String {
        showsAgentBadge ? Localized.Diff.includesAgentMarks(label: reviewChrome.accessibilityLabel) : reviewChrome.accessibilityLabel
    }

    var body: some View {
        let colors = appThemes.colors(for: colorScheme)
        HStack(spacing: 8) {
            HStack(alignment: .top, spacing: 8) {
                if hasConflict {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundStyle(.red)
                        .jayjayFont(11)
                } else {
                    hunk.hunkType.badge(colorScheme, colors: colors)
                        .jayjayFont(18)
                        .accessibilityLabel(hunk.hunkType.label)
                        .help(hunk.hunkType.label)
                }

                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 6) {
                        Text(URL(fileURLWithPath: hunk.path).lastPathComponent)
                            .jayjayFont(12, weight: .medium)
                            .lineLimit(1)
                            .opacity(showsReviewedStyle ? 0.5 : 1)
                        if hunk.isSubmodulePlaceholder {
                            Text(Localized.Repo.submodule)
                                .jayjayFont(9, weight: .semibold)
                                .foregroundStyle(.secondary)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color.secondary.opacity(0.12), in: Capsule())
                        } else if hunk.isGitLfsPlaceholder {
                            Text(Localized.Repo.lfs)
                                .jayjayFont(9, weight: .semibold)
                                .foregroundStyle(.secondary)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color.secondary.opacity(0.12), in: Capsule())
                        }
                        if showsAgentBadge {
                            Text(Localized.Repo.agent)
                                .jayjayFont(9, weight: .semibold)
                                .foregroundStyle(.secondary)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color.secondary.opacity(0.12), in: Capsule())
                                .help(Localized.Repo.includesChangesMarkedByAnAgent)
                                .accessibilityIdentifier(AID.FileList.agentReviewed(hunk.path))
                        }
                        if noteCount > 0 {
                            HStack(alignment: .firstTextBaseline, spacing: 3) {
                                Image(systemName: "note.text")
                                    .jayjayFont(8)
                                Text("\(noteCount)")
                                    .jayjayFont(9, weight: .semibold)
                                    .accessibilityIdentifier(AID.ReviewNote.fileCount(path: hunk.path, count: noteCount))
                            }
                            .foregroundStyle(colors.noteAccent)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(colors.noteAccent.opacity(0.12), in: Capsule())
                            .help(noteCount.reviewNoteCountLabel)
                        }
                    }
                    // A move truncates each side on its own, so the arrow and the destination never fall into the elided middle.
                    if let movedFrom {
                        HStack(spacing: 4) {
                            Text(movedFrom).truncationMode(.middle)
                            Text(verbatim: "→").fixedSize()
                            Text(hunk.path).truncationMode(.middle)
                        }
                        .jayjayFont(10, design: .monospaced)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .accessibilityElement(children: .ignore)
                        .accessibilityLabel(Localized.Diff.movedFromTo(kind: hunk.hunkType.label, old: movedFrom, new: hunk.path))
                    } else if let pathDetail {
                        Text(pathDetail)
                            .jayjayFont(10, design: .monospaced)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                            .accessibilityLabel(hunk.path)
                    }
                }

                Spacer(minLength: 0)
            }

            if showReview {
                Button {
                    onToggleReview?()
                } label: {
                    Image(systemName: reviewChrome.systemImage)
                        .foregroundStyle(reviewChrome.tint)
                        .jayjayFont(14)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier(AID.FileList.review(hunk.path))
                .accessibilityLabel(reviewAccessibilityLabel)
            }
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 6)
        .frame(minHeight: PaneLayout.fileRowHeight(scale: typography.interfaceScale))
        .help(hunk.oldPath.map { "\($0) → \(hunk.path)" } ?? hunk.path)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(isSelected ? Color.accentColor.opacity(isPaneActive ? 0.28 : 0.14) : .clear)
        )
    }
}
