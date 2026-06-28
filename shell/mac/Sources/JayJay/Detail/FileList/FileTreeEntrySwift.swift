import Foundation
import JayJayCore

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
