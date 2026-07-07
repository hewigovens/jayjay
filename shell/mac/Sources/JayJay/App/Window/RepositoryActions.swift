import AppKit
import Foundation

protocol FileRevealingWorkspace {
    @discardableResult
    func selectFile(_ fullPath: String?, inFileViewerRootedAtPath rootFullPath: String) -> Bool
    func activateFileViewerSelecting(_ fileURLs: [URL])
}

extension NSWorkspace: FileRevealingWorkspace {}

enum RepositoryActions {
    static let captureShowInFinderPasteboardEnvironmentKey = "JAYJAY_CAPTURE_SHOW_IN_FINDER_PASTEBOARD"

    static func showInFinder(repoPath: String) {
        showInFinder(repoPath: repoPath, path: nil)
    }

    static func showInFinder(repoPath: String, path: String?) {
        let targetURL = fileViewerSelectionURL(repoPath: repoPath, path: path)
        if ProcessInfo.processInfo.environment[captureShowInFinderPasteboardEnvironmentKey] == "1" {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(targetURL.path, forType: .string)
            return
        }
        revealInFinder(selectionURL: targetURL)
    }

    @discardableResult
    static func revealInFinder(
        selectionURL: URL,
        workspace: any FileRevealingWorkspace = NSWorkspace.shared
    ) -> Bool {
        if workspace.selectFile(
            selectionURL.path,
            inFileViewerRootedAtPath: fileViewerRootURL(selectionURL: selectionURL).path
        ) {
            return true
        }
        workspace.activateFileViewerSelecting([selectionURL])
        return false
    }

    static func fileViewerRootURL(selectionURL: URL) -> URL {
        selectionURL.deletingLastPathComponent()
    }

    static func fileViewerSelectionURL(
        repoPath: String,
        path: String?,
        fileManager: FileManager = .default
    ) -> URL {
        let repoURL = URL(fileURLWithPath: repoPath).standardizedFileURL
        guard let path, !path.isEmpty else {
            return repoURL
        }

        // Reveal the deepest existing ancestor of the target inside the repo,
        // falling back to the repo root when nothing along the way exists.
        var candidate = repoURL.appendingPathComponent(path).standardizedFileURL
        while candidate.path == repoURL.path || candidate.path.hasPrefix("\(repoURL.path)/") {
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
