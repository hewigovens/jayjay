import AppKit
import Foundation

extension RepoViewModel {
    var isRepoWindowActive: Bool {
        guard NSApp.isActive,
              let window = activeRepoWindow,
              window.isVisible,
              !window.isMiniaturized,
              let activePath = window.representedURL?.standardizedFileURL.path
        else {
            return false
        }
        return activePath == URL(fileURLWithPath: repoPath).standardizedFileURL.path
    }

    private var activeRepoWindow: NSWindow? {
        if let window = NSApp.keyWindow, window.representedURL != nil {
            return window
        }
        if NSApp.keyWindow is NSPanel {
            return NSApp.mainWindow
        }
        return nil
    }
}
