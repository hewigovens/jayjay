import AppKit
@testable import JayJay
import XCTest

@MainActor
final class EditUndoRouterTests: XCTestCase {
    private final class FocusableView: NSView {
        override var acceptsFirstResponder: Bool {
            true
        }
    }

    func testRepoWindowsKeepTextUndoForFocusedTextAndOtherWindowsKeepTheirOwnHistory() throws {
        _ = NSApplication.shared
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 300, height: 200), styleMask: [.titled], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        let text = NSTextView(frame: NSRect(x: 0, y: 0, width: 100, height: 100))
        text.allowsUndo = true
        let list = FocusableView(frame: NSRect(x: 100, y: 0, width: 100, height: 100))
        window.contentView?.addSubview(text)
        window.contentView?.addSubview(list)

        window.representedURL = URL(fileURLWithPath: "/tmp/repo")
        window.makeFirstResponder(text)
        XCTAssertFalse(EditUndoRouter.acceptsRepoSteps(window), "an empty text history must never fall through to repository undo")
        XCTAssertFalse(EditUndoRouter.LocalHistory(window: window).canUndo)

        let manager = try XCTUnwrap(text.undoManager)
        manager.groupsByEvent = false
        manager.beginUndoGrouping()
        manager.registerUndo(withTarget: text) { _ in }
        manager.endUndoGrouping()
        let history = EditUndoRouter.LocalHistory(window: window)
        XCTAssertTrue(history.canUndo)
        XCTAssertEqual(history.undoTitle, manager.undoMenuItemTitle)
        XCTAssertFalse(EditUndoRouter.acceptsRepoSteps(window))

        window.representedURL = nil
        window.makeFirstResponder(list)
        XCTAssertFalse(EditUndoRouter.acceptsRepoSteps(window))
        XCTAssertTrue(EditUndoRouter.LocalHistory(window: window).canUndo, "a sheet or editor window keeps its own history, e.g. merge hunk picks")

        window.representedURL = URL(fileURLWithPath: "/tmp/repo")
        XCTAssertTrue(EditUndoRouter.acceptsRepoSteps(window))
        XCTAssertFalse(EditUndoRouter.LocalHistory(window: window).canUndo, "in a repo window ⌘Z off a text input steps jj operations")
    }
}
