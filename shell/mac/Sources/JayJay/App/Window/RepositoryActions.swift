import AppKit
import Foundation

enum RepositoryActions {
    static func showInFinder(repoPath: String) {
        showInFinder(repoPath: repoPath, path: nil)
    }

    static func showInFinder(repoPath: String, path: String?) {
        let targetURL = fileViewerSelectionURL(repoPath: repoPath, path: path)
        NSWorkspace.shared.activateFileViewerSelecting([targetURL])
    }

    static func fileViewerSelectionURL(
        repoPath: String,
        path: String?,
        fileManager: FileManager = .default
    ) -> URL {
        let repoURL = URL(fileURLWithPath: repoPath)
        guard let path, !path.isEmpty else {
            return repoURL
        }

        // Reveal the deepest existing ancestor of the target inside the repo,
        // falling back to the repo root when nothing along the way exists.
        var candidate = repoURL.appendingPathComponent(path)
        while candidate.path.hasPrefix(repoURL.path) {
            if fileManager.fileExists(atPath: candidate.path) {
                return candidate
            }
            let parent = candidate.deletingLastPathComponent()
            if parent.path == candidate.path {
                break
            }
            candidate = parent
        }

        return repoURL
    }
}
