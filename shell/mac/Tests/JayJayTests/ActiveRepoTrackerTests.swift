import AppKit
@testable import JayJay
import XCTest

@MainActor
final class ActiveRepoTrackerTests: XCTestCase {
    private final class Handler: RepositoryMenuHandler {
        func showCommandPalette() {}
        func showUndo() {}
        func showBookmarkManager() {}
        func showNewWorkspace() {}
    }

    func testRepositoryListWindowClearsTheActiveRepository() throws {
        _ = NSApplication.shared
        let suite = "ActiveRepoTrackerTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let handler = Handler()
        let tracker = ActiveRepoTracker.shared
        tracker.register(repoPath: "/tmp/repo", settings: AppSettings(defaults: defaults), handler: handler)

        let repoList = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 300, height: 200), styleMask: [.titled], backing: .buffered, defer: false)
        repoList.isReleasedWhenClosed = false
        repoList.identifier = NSUserInterfaceItemIdentifier(AppWindows.repoList)
        NotificationCenter.default.post(name: NSWindow.didBecomeKeyNotification, object: repoList)
        XCTAssertNil(tracker.repoPath, "the repository list has no repository for the Repository menu to act on")
        XCTAssertNil(tracker.handler)

        let repoWindow = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 300, height: 200), styleMask: [.titled], backing: .buffered, defer: false)
        repoWindow.isReleasedWhenClosed = false
        repoWindow.representedURL = URL(fileURLWithPath: "/tmp/repo")
        NotificationCenter.default.post(name: NSWindow.didBecomeKeyNotification, object: repoWindow)
        XCTAssertEqual(tracker.repoPath, "/tmp/repo")
        XCTAssertTrue(tracker.handler === handler)
    }
}
