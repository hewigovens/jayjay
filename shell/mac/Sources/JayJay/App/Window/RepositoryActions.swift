import AppKit
import Foundation

enum RepositoryActions {
    static func showInFinder(repoPath: String) {
        let repoURL = URL(fileURLWithPath: repoPath)
        NSWorkspace.shared.activateFileViewerSelecting([repoURL])
    }
}
