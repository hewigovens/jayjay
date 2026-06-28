import JayJayCore

func reviewNoteCountLabel(_ count: Int) -> String {
    "\(count) review \(count == 1 ? "note" : "notes")"
}

import SwiftUI

struct FileRow: View {
    let hunk: DiffHunk
    let isSelected: Bool
    var showReview: Bool = false
    var isReviewed: Bool = false
    var noteCount: Int = 0
    var hasConflict: Bool = false
    var onToggleReview: (() -> Void)?

    var showsReviewedStyle: Bool {
        showReview && isReviewed
    }

    var body: some View {
        HStack(spacing: 8) {
            if showReview {
                Button {
                    onToggleReview?()
                } label: {
                    Image(systemName: showsReviewedStyle ? "checkmark.circle.fill" : "circle")
                        .foregroundStyle(showsReviewedStyle ? Color.green : Color.secondary.opacity(0.4))
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
                        .help(reviewNoteCountLabel(noteCount))
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

// MARK: - Tree file list

struct TreeFileList<RowContent: View>: View {
    let filteredDiff: [DiffHunk]
    let commitId: String
    let fileRowView: (DiffHunk) -> RowContent
    @State private var treeEntries: [FileTreeEntry] = []
    @State private var collapsedDirs: Set<String> = []

    var body: some View {
        List {
            ForEach(visibleEntries, id: \.path) { entry in
                if let hunkIdx = entry.hunkIndex, Int(hunkIdx) < filteredDiff.count {
                    let hunk = filteredDiff[Int(hunkIdx)]
                    fileRowView(hunk)
                        .padding(.leading, CGFloat(entry.depth) * 12)
                        .tag(hunk.path)
                } else {
                    folderRow(entry)
                }
            }
        }
        .listStyle(.plain)
        .scrollIndicators(.never)
        .id(commitId)
        .onAppear { rebuildTree() }
        .onChange(of: filteredDiff.map(\.path)) { rebuildTree() }
    }

    private var visibleEntries: [FileTreeEntry] {
        guard !collapsedDirs.isEmpty else { return treeEntries }
        return treeEntries.filter { entry in
            !collapsedDirs.contains(where: { entry.path.hasPrefix("\($0)/") })
        }
    }

    @ViewBuilder
    private func folderRow(_ entry: FileTreeEntry) -> some View {
        let isCollapsed = collapsedDirs.contains(entry.path)
        Button {
            if isCollapsed {
                collapsedDirs.remove(entry.path)
            } else {
                collapsedDirs.insert(entry.path)
            }
        } label: {
            HStack(spacing: 4) {
                Image(systemName: isCollapsed ? "chevron.right" : "chevron.down")
                    .foregroundStyle(.tertiary)
                    .jayjayFont(9)
                    .frame(width: 10)
                Image(systemName: "folder").foregroundStyle(.secondary).jayjayFont(11)
                Text(entry.name).jayjayFont(12, weight: .medium)
                Spacer(minLength: 0)
            }
            .contentShape(Rectangle())
            .padding(.leading, CGFloat(entry.depth) * 12)
        }
        .buttonStyle(.plain)
    }

    private func rebuildTree() {
        treeEntries = buildFileTree(paths: filteredDiff.map(\.path))
        // Drop collapsed dirs that no longer exist in the new tree, otherwise a leftover prefix from a previous change can hide files in this one.
        let validDirs = Set(treeEntries.compactMap { $0.hunkIndex == nil ? $0.path : nil })
        collapsedDirs.formIntersection(validDirs)
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
