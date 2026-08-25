import JayJayCore
import SwiftUI

struct TreeFileList<RowContent: View>: View {
    let filteredDiff: [DiffHunk]
    let commitId: String
    let fileRowView: (DiffHunk) -> RowContent
    @State private var treeEntries: [FileTreeEntry] = []
    @State private var collapsedDirs: Set<String> = []

    var body: some View {
        List {
            ForEach(visibleEntries, id: \.path) { entry in
                Group {
                    if let hunkIdx = entry.hunkIndex, Int(hunkIdx) < filteredDiff.count {
                        let hunk = filteredDiff[Int(hunkIdx)]
                        fileRowView(hunk)
                            .padding(.leading, CGFloat(entry.depth) * 12)
                            .tag(hunk.path)
                    } else {
                        folderRow(entry)
                    }
                }
                .listRowInsets(FileRow.listInsets)
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
