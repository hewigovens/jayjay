import JayJayCore
import SwiftUI

struct FileRow: View {
    let hunk: DiffHunk
    let isSelected: Bool
    var showReview: Bool = false
    var isReviewed: Bool = false
    var hasConflict: Bool = false
    var onToggleReview: (() -> Void)?

    var body: some View {
        HStack(spacing: 8) {
            if showReview {
                Button {
                    onToggleReview?()
                } label: {
                    Image(systemName: isReviewed ? "checkmark.circle.fill" : "circle")
                        .foregroundStyle(isReviewed ? Color.green : Color.secondary.opacity(0.4))
                        .jayjayFont(14)
                }
                .buttonStyle(.plain)
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
                        .opacity(isReviewed ? 0.5 : 1)
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
            case .modified: return Color.orange
            case .renamed: return Color.blue
        }
    }
}

// MARK: - File tree (via Rust)

struct FileTreeEntrySwift: Identifiable {
    let id = UUID()
    let name: String
    let path: String
    let depth: Int
    let hunk: DiffHunk?
}

extension [DiffHunk] {
    func buildTree() -> [FileTreeEntrySwift] {
        let paths = map(\.path)
        let entries = buildFileTree(paths: paths)
        return entries.map { entry in
            FileTreeEntrySwift(
                name: entry.name,
                path: entry.path,
                depth: Int(entry.depth),
                hunk: entry.hunkIndex.flatMap { idx in
                    Int(idx) < self.count ? self[Int(idx)] : nil
                }
            )
        }
    }
}
