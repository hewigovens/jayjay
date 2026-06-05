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

        let targetURL = repoURL.appendingPathComponent(path)
        if fileManager.fileExists(atPath: targetURL.path) {
            return targetURL
        }

        var candidate = targetURL.deletingLastPathComponent()
        while candidate.path.hasPrefix(repoURL.path) {
            if fileManager.fileExists(atPath: candidate.path) {
                return URL(fileURLWithPath: candidate.path)
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
