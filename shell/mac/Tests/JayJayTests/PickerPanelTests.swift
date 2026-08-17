import AppKit
@testable import JayJay
import SwiftUI
import XCTest

@MainActor
final class PickerPanelTests: XCTestCase {
    func testHostWindowCloseDismissesPanelAndReleasesContent() {
        _ = NSApplication.shared
        let host = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 400, height: 300),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        let anchor = NSView(frame: NSRect(x: 20, y: 250, width: 120, height: 30))
        host.contentView = anchor
        let panel = PickerPanel()
        panel.animationBehavior = .none
        panel.show(
            under: anchor,
            size: NSSize(width: 280, height: 200),
            content: Text("Workspace picker")
        )

        XCTAssertTrue(panel.isVisible)
        XCTAssertNotNil(panel.contentViewController)
        XCTAssertTrue(host.childWindows?.contains(panel) == true)

        // Posting the notification exercises the production close observer without invoking AppKit window-transform animation inside the app-hosted XCTest process.
        NotificationCenter.default.post(name: NSWindow.willCloseNotification, object: host)

        XCTAssertFalse(panel.isVisible)
        XCTAssertNil(panel.contentViewController)
        XCTAssertFalse(host.childWindows?.contains(panel) == true)
    }
}
