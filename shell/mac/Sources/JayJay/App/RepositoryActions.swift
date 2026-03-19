import AppKit
import Foundation

enum RepositoryActions {
    static var isVSCodeInstalled: Bool {
        appURL(for: .visualStudioCode) != nil
    }

    static var isGhosttyInstalled: Bool {
        appURL(for: .ghostty) != nil
    }

    static func showInFinder(repoPath: String) {
        let repoURL = URL(fileURLWithPath: repoPath)
        NSWorkspace.shared.activateFileViewerSelecting([repoURL])
    }

    static func openInVSCode(repoPath: String) {
        let repoURL = URL(fileURLWithPath: repoPath)
        guard let appURL = appURL(for: .visualStudioCode) else {
            NSSound.beep()
            return
        }

        let configuration = NSWorkspace.OpenConfiguration()
        NSWorkspace.shared.open([repoURL], withApplicationAt: appURL, configuration: configuration) { _, error in
            if error != nil {
                NSSound.beep()
            }
        }
    }

    static func openInGhostty(repoPath: String) {
        guard let appURL = appURL(for: .ghostty) else {
            NSSound.beep()
            return
        }

        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/open")
        process.arguments = [
            "-na", appURL.path,
            "--args",
            "--working-directory=\(repoPath)"
        ]

        do {
            try process.run()
        } catch {
            NSSound.beep()
        }
    }

    private static func appURL(for app: ExternalApp) -> URL? {
        NSWorkspace.shared.urlForApplication(withBundleIdentifier: app.bundleIdentifier)
    }
}

private enum ExternalApp {
    case visualStudioCode
    case ghostty

    var bundleIdentifier: String {
        switch self {
        case .visualStudioCode:
            "com.microsoft.VSCode"
        case .ghostty:
            "com.mitchellh.ghostty"
        }
    }
}
