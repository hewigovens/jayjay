import JayJayCore

/// Shared by the tree list and keyboard navigation so key repeat doesn't rebuild the tree on every press.
final class FileTreeCache {
    private var paths: [String] = []
    private var cached: [FileTreeEntry] = []

    func entries(for paths: [String]) -> [FileTreeEntry] {
        if paths != self.paths {
            self.paths = paths
            cached = buildFileTree(paths: paths)
        }
        return cached
    }
}
