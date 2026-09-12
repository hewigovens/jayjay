import AppKit
@testable import JayJay
import JayJayCore
@testable import JayJayDiffUI
import SwiftUI
import XCTest

@MainActor
final class DiffSectionInteractionTests: XCTestCase {
    func testCardBorderDoesNotInterceptImageDividerDrag() throws {
        let content = DiffContent(content: nil, preview: .image(path: "/unavailable-preview.png"))
        let hunk = DiffHunk(
            path: "preview.png",
            oldPath: nil,
            old: content,
            new: content,
            hunkType: .modified,
            supportsConflictEditor: false,
            supportsFileEditor: false,
            reviewIdentity: "image",
            projection: nil
        )
        let section = DiffSection(
            hunk: hunk,
            rev: nil,
            repo: nil,
            actions: nil,
            isWorkingCopy: false,
            diffStore: DiffStore(),
            reviewStore: nil,
            noteEditor: .constant(nil)
        )
        let settings = try AppSettings(defaults: XCTUnwrap(UserDefaults(suiteName: "DiffSectionInteractionTests")))
        let hosting = NSHostingView(rootView: section.environment(settings))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 420, height: 400), styleMask: [.titled], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer { window.close() }
        window.contentView = hosting
        window.orderBack(nil)
        hosting.layoutSubtreeIfNeeded()
        for _ in 0 ..< 5 {
            RunLoop.main.run(until: Date().addingTimeInterval(0.05))
        }
        let divider = try XCTUnwrap(findDivider(in: hosting))
        let start = divider.convert(NSPoint(x: divider.bounds.midX, y: divider.bounds.midY), to: nil)
        for (type, dx) in [(NSEvent.EventType.leftMouseDown, 0.0), (.leftMouseDragged, 40.0), (.leftMouseUp, 40.0)] {
            try window.sendEvent(XCTUnwrap(NSEvent.mouseEvent(
                with: type, location: NSPoint(x: start.x + dx, y: start.y), modifierFlags: [],
                timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: window.windowNumber,
                context: nil, eventNumber: 0, clickCount: 1, pressure: 0
            )))
        }
        for _ in 0 ..< 5 {
            RunLoop.main.run(until: Date().addingTimeInterval(0.05))
        }
        let finish = divider.convert(NSPoint(x: divider.bounds.midX, y: divider.bounds.midY), to: nil)
        XCTAssertEqual(finish.x, start.x + 40, accuracy: 1)
    }

    private func findDivider(in view: NSView) -> ImageDiffDivider.MouseView? {
        if let divider = view as? ImageDiffDivider.MouseView {
            return divider
        }
        return view.subviews.lazy.compactMap { self.findDivider(in: $0) }.first
    }
}
