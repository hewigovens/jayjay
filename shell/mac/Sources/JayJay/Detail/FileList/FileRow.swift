import JayJayCore
import SwiftUI

struct FileRow: View {
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
        showsAgentBadge ? "\(reviewChrome.accessibilityLabel), includes agent marks" : reviewChrome.accessibilityLabel
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
                .accessibilityLabel(reviewAccessibilityLabel)
            }

            HStack(alignment: .firstTextBaseline, spacing: 8) {
                if hasConflict {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundStyle(.red)
                        .jayjayFont(11)
                } else {
                    Text(statusLetter)
                        .jayjayFont(.secondary)
                        .fontWeight(.bold)
                        .fontDesign(.monospaced)
                        .foregroundStyle(color)
                        .frame(width: 20, height: 20)
                        .background(color.opacity(0.12), in: RoundedRectangle(cornerRadius: 5))
                        .accessibilityLabel(hunk.hunkType.label)
                }

                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 6) {
                        Text(URL(fileURLWithPath: hunk.path).lastPathComponent)
                            .jayjayFont(.body)
                            .fontWeight(.medium)
                            .lineLimit(2)
                            .truncationMode(.middle)
                            .fixedSize(horizontal: false, vertical: true)
                            .layoutPriority(1)
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
                        if showsAgentBadge {
                            Text("Agent")
                                .jayjayFont(9, weight: .semibold)
                                .foregroundStyle(.secondary)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color.secondary.opacity(0.12), in: Capsule())
                                .help("Includes changes marked by an agent")
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
                            .foregroundStyle(.orange)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.orange.opacity(0.12), in: Capsule())
                            .help(noteCount.reviewNoteCountLabel)
                        }
                    }

                    if hunk.hunkType == .renamed, let oldPath = hunk.oldPath {
                        HStack(alignment: .firstTextBaseline, spacing: 3) {
                            Text(oldPath)
                                .strikethrough()
                            Image(systemName: "arrow.right")
                                .imageScale(.small)
                            Text(hunk.path)
                        }
                        .jayjayFont(.secondary)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                    } else {
                        Text((hunk.path as NSString).deletingLastPathComponent.isEmpty
                            ? "Repository root" : (hunk.path as NSString).deletingLastPathComponent)
                            .jayjayFont(.secondary)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }
                }
                Spacer(minLength: 0)
            }
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 9)
        .help(hunk.path)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(isSelected ? Color.accentColor.opacity(isPaneActive ? 0.28 : 0.14) : .clear)
        )
    }

    private var statusLetter: String {
        switch hunk.hunkType {
            case .added: "A"
            case .removed: "D"
            case .modified: "M"
            case .renamed: "R"
        }
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
