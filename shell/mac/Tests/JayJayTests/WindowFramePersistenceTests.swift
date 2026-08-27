import AppKit
@testable import JayJay
import SwiftUI
import XCTest

@MainActor
final class WindowFramePersistenceTests: XCTestCase {
    func testSavedFrameIsAppliedBeforeTheWindowIsShown() throws {
        _ = NSApplication.shared
        let (defaults, key) = try makeIsolatedDefaults()
        let visible = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        let saved = NSRect(x: visible.minX + 40, y: visible.minY + 40, width: 640, height: 400)
        WindowFrameStore.save(saved, key: key, defaults: defaults)

        let window = NSWindow(
            contentRect: NSRect(x: visible.minX + 300, y: visible.minY + 300, width: 300, height: 200),
            styleMask: [.titled, .resizable],
            backing: .buffered,
            defer: false
        )
        window.isReleasedWhenClosed = false
        defer { window.close() }
        window.contentView = NSHostingView(rootView: Color.clear.background(WindowFramePersistence(key: key, defaults: defaults)))
        window.layoutIfNeeded()

        XCTAssertFalse(window.isVisible)
        XCTAssertEqual(window.frame, saved, "the saved frame must be in place before the window is ordered front")
    }

    func testObserversStopAtClose() throws {
        _ = NSApplication.shared
        let (defaults, key) = try makeIsolatedDefaults()
        let window = NSWindow(
            contentRect: NSRect(x: 100, y: 100, width: 300, height: 200),
            styleMask: [.titled, .resizable],
            backing: .buffered,
            defer: false
        )
        window.isReleasedWhenClosed = false
        defer { window.close() }
        window.contentView = NSHostingView(rootView: Color.clear.background(WindowFramePersistence(key: key, defaults: defaults)))
        window.layoutIfNeeded()
        window.orderFront(nil)
        RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05))

        let moved = NSRect(x: 140, y: 120, width: 320, height: 210)
        window.setFrame(moved, display: false)
        XCTAssertEqual(WindowFrameStore.frame(key: key, defaults: defaults), moved, "a settled window persists its frame")

        window.close()
        window.setFrame(NSRect(x: 10, y: 10, width: 200, height: 150), display: false)
        XCTAssertEqual(WindowFrameStore.frame(key: key, defaults: defaults), moved, "a closed window must stop reporting frames")
    }

    func testFrameChangesAfterTheRunLoopIdlesAreTheUsers() throws {
        _ = NSApplication.shared
        let (defaults, key) = try makeIsolatedDefaults()
        let visible = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        let saved = NSRect(x: visible.minX + 40, y: visible.minY + 40, width: 640, height: 400)
        WindowFrameStore.save(saved, key: key, defaults: defaults)
        let window = NSWindow(contentRect: saved, styleMask: [.titled, .resizable], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer { window.close() }
        window.contentView = NSHostingView(rootView: Color.clear.background(WindowFramePersistence(key: key, defaults: defaults)))
        window.layoutIfNeeded()
        window.orderFront(nil)

        let nudged = saved.offsetBy(dx: 30, dy: 0)
        window.setFrame(nudged, display: false)
        XCTAssertEqual(window.frame, saved, "a move in the attaching run-loop turn is SwiftUI's placement and is undone")

        RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05))
        window.setFrame(nudged, display: false)
        XCTAssertEqual(window.frame, nudged, "a move after the run loop idled is the user's")
        XCTAssertEqual(WindowFrameStore.frame(key: key, defaults: defaults), nudged)
    }

    private func makeIsolatedDefaults() throws -> (UserDefaults, String) {
        let suite = "WindowFramePersistenceTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        addTeardownBlock { defaults.removePersistentDomain(forName: suite) }
        return (defaults, "scene")
    }
}
