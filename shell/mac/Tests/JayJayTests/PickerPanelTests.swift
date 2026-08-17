import AppKit
@testable import JayJay
import SwiftUI
import XCTest

@MainActor
final class PickerPanelTests: XCTestCase {
    private func makeHost() -> (NSWindow, NSView) {
        _ = NSApplication.shared
        let host = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 400, height: 300),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        host.isReleasedWhenClosed = false
        host.animationBehavior = .none
        let anchor = NSView(frame: NSRect(x: 20, y: 250, width: 120, height: 30))
        host.contentView = anchor
        return (host, anchor)
    }

    private func show(_ panel: PickerPanel, under anchor: NSView) {
        panel.animationBehavior = .none
        panel.show(under: anchor, size: NSSize(width: 280, height: 200), content: Text("Workspace picker"))
    }

    func testPanelLivesAndDiesWithItsHostWindow() {
        let (host, anchor) = makeHost()
        let panel = PickerPanel()
        show(panel, under: anchor)
        XCTAssertTrue(panel.isVisible)
        XCTAssertTrue(host.childWindows?.contains(panel) == true)

        host.close()
        XCTAssertFalse(panel.isVisible)
        XCTAssertNil(panel.contentViewController)

        show(panel, under: anchor)
        XCTAssertTrue(panel.isVisible, "a panel closed with its host must still open again")
        panel.dismiss()

        show(panel, under: NSView(frame: NSRect(x: 0, y: 0, width: 120, height: 30)))
        XCTAssertFalse(panel.isVisible, "no host window, no panel")
    }

    func testOnlyFocusLossArmsTheToggleGuard() {
        let (host, anchor) = makeHost()
        let panel = PickerPanel()
        show(panel, under: anchor)

        panel.dismiss()
        XCTAssertFalse(panel.wasJustDismissed)

        show(panel, under: anchor)
        panel.dismissOnFocusLoss()
        XCTAssertTrue(panel.wasJustDismissed)
        host.close()
    }
}
