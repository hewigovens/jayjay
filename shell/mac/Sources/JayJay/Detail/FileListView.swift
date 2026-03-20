import SwiftUI
import JayJayBindings

struct FileRow: View {
    let hunk: DiffHunk
    let isSelected: Bool
    var showReview: Bool = false
    var isReviewed: Bool = false
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

            Circle()
                .fill(color)
                .frame(width: 6, height: 6)

            VStack(alignment: .leading, spacing: 2) {
                Text(URL(fileURLWithPath: hunk.path).lastPathComponent)
                    .jayjayFont(12, weight: .medium)
                    .lineLimit(1)
                    .opacity(isReviewed ? 0.5 : 1)

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
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .background(isSelected ? color.opacity(0.14) : Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private var color: Color {
        switch hunk.hunkType {
        case .added: .green
        case .removed: .red
        case .modified: .orange
        case .renamed: .blue
        }
    }
}

// MARK: - File tree

struct FileTreeEntry: Identifiable {
    let id = UUID()
    let name: String
    let path: String
    let depth: Int
    let hunk: DiffHunk?
}

struct FileTreeNode {
    var name: String
    var children: [(String, FileTreeNode)]
    var hunk: DiffHunk?

    static func build(from hunks: [DiffHunk]) -> FileTreeNode {
        var root = FileTreeNode(name: "", children: [], hunk: nil)
        for hunk in hunks {
            let components = hunk.path.split(separator: "/").map(String.init)
            root.insert(components: components, hunk: hunk)
        }
        root.collapse()
        return root
    }

    mutating func insert(components: [String], hunk: DiffHunk) {
        guard let first = components.first else { return }
        if components.count == 1 {
            children.append((first, FileTreeNode(name: first, children: [], hunk: hunk)))
        } else {
            if let idx = children.firstIndex(where: { $0.0 == first }) {
                children[idx].1.insert(components: Array(components.dropFirst()), hunk: hunk)
            } else {
                var child = FileTreeNode(name: first, children: [], hunk: nil)
                child.insert(components: Array(components.dropFirst()), hunk: hunk)
                children.append((first, child))
            }
        }
    }

    mutating func collapse() {
        for i in children.indices {
            children[i].1.collapse()
        }
        if hunk == nil && children.count == 1 && children[0].1.hunk == nil {
            let child = children[0].1
            name = name.isEmpty ? children[0].0 : "\(name)/\(children[0].0)"
            children = child.children
        }
    }

    func flattenedEntries(depth: Int = 0) -> [FileTreeEntry] {
        var result: [FileTreeEntry] = []
        let sortedChildren = children.sorted { $0.1.hunk == nil && $1.1.hunk != nil }
        for (key, child) in sortedChildren {
            if child.hunk != nil {
                result.append(FileTreeEntry(name: key, path: child.hunk!.path, depth: depth, hunk: child.hunk))
            } else {
                let dirName = child.name.isEmpty ? key : child.name
                result.append(FileTreeEntry(name: dirName, path: "dir://\(dirName)/\(depth)", depth: depth, hunk: nil))
                result.append(contentsOf: child.flattenedEntries(depth: depth + 1))
            }
        }
        return result
    }
}
