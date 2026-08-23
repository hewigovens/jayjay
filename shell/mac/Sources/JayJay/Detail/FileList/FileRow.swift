import JayJayCore
import SwiftUI

struct FileRow: View {
    let hunk: DiffHunk
    let isSelected: Bool
    var showReview: Bool = false
    var reviewRollup: ReviewFileRollup = .unreviewed
    var noteCount: Int = 0
    var hasConflict: Bool = false
    var onToggleReview: (() -> Void)?

    var reviewChrome: FileRowReviewChrome {
        FileRowReviewChrome.chrome(showReview: showReview, rollup: reviewRollup)
    }

    var showsReviewedStyle: Bool {
        reviewChrome == .reviewed
    }

    var body: some View {
        HStack(spacing: 8) {
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
                .accessibilityLabel(reviewChrome.accessibilityLabel)
            }

            if hasConflict {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.red)
                    .jayjayFont(11)
            } else {
                Circle()
                    .fill(color)
                    .frame(width: 6, height: 6)
            }

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(URL(fileURLWithPath: hunk.path).lastPathComponent)
                        .jayjayFont(12, weight: .medium)
                        .lineLimit(1)
                        .opacity(showsReviewedStyle ? 0.5 : 1)
                    if hunk.isSubmodulePlaceholder {
                        Text("Submodule")
                            .jayjayFont(9, weight: .semibold)
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.secondary.opacity(0.12), in: Capsule())
                    } else if hunk.isGitLfsPlaceholder {
                        Text("LFS")
                            .jayjayFont(9, weight: .semibold)
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.secondary.opacity(0.12), in: Capsule())
                    }
                    if noteCount > 0 {
                        HStack(spacing: 3) {
                            Image(systemName: "note.text")
                                .jayjayFont(8)
                            Text("\(noteCount)")
                                .jayjayFont(9, weight: .semibold)
                                .accessibilityIdentifier(AID.ReviewNote.fileCount(path: hunk.path, count: noteCount))
                        }
                        .foregroundStyle(.orange)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Color.orange.opacity(0.12), in: Capsule())
                        .help(noteCount.reviewNoteCountLabel)
                    }
                }

                if hunk.hunkType == .renamed, let oldPath = hunk.oldPath {
                    HStack(spacing: 3) {
                        Text(oldPath)
                            .strikethrough()
                        Image(systemName: "arrow.right")
                            .imageScale(.small)
                        Text(hunk.path)
                    }
                    .jayjayFont(9, design: .monospaced)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
                } else {
                    Text(hunk.path)
                        .jayjayFont(9, design: .monospaced)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 6)
        .padding(.vertical, 6)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(isSelected ? Color.accentColor.opacity(0.14) : .clear)
        )
    }

    private var color: Color {
        if hunk.isSubmodulePlaceholder {
            return Color.blue
        }
        if hunk.isGitLfsPlaceholder {
            return Color.purple
        }
        switch hunk.hunkType {
            case .added: return Color.green
            case .removed: return Color.red
            case .modified: return FileStatusColors.modified
            case .renamed: return Color.blue
        }
    }
}
