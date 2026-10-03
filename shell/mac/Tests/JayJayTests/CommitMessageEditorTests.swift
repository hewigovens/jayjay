import AppKit
@testable import JayJay
import SwiftUI
import XCTest

@MainActor
final class CommitMessageEditorTests: XCTestCase {
    func testTypingPastSummaryWidthKeepsInsertionPointVisible() async throws {
        let draft = Draft()
        let focus = KeyboardFocus()
        let host = NSHostingView(rootView: CommitMessageEditor(
            summary: Binding(get: { draft.summary }, set: { draft.summary = $0 }),
            details: .constant(""), participatesInPaneNavigation: true
        ).environment(focus).environment(\.jayjayFontSize, 10))
        let window = NSWindow(
            contentRect: CGRect(x: 0, y: 0, width: 250, height: 180),
            styleMask: [.titled], backing: .buffered, defer: false
        )
        window.isReleasedWhenClosed = false
        defer { window.close() }
        window.contentView = host
        window.makeKeyAndOrderFront(nil)
        let layoutDeadline = Date(timeIntervalSinceNow: 2)
        while summaryField(in: host) == nil, Date() < layoutDeadline {
            try await Task.sleep(for: .milliseconds(50))
        }
        let field = try XCTUnwrap(summaryField(in: host))
        XCTAssertTrue(window.makeFirstResponder(field))
        let editor = try XCTUnwrap(window.firstResponder as? NSTextView)
        editor.setSelectedRange(NSRange(location: 0, length: 0))
        let text = String(repeating: "abcdefghijklmnopqrstuvwxyz ", count: 3) + "🐦é"
        let typingInterval = Duration.milliseconds(50)
        let caretTolerance: CGFloat = 1
        var expected = ""
        for character in text {
            let key = String(character)
            let event = try XCTUnwrap(NSEvent.keyEvent(
                with: .keyDown, location: .zero, modifierFlags: [],
                timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: window.windowNumber,
                context: nil, characters: key, charactersIgnoringModifiers: key,
                isARepeat: false, keyCode: 0
            ))
            editor.keyDown(with: event)
            expected += key
            try await Task.sleep(for: typingInterval)
            XCTAssertEqual(draft.summary, expected)
            XCTAssertEqual(editor.selectedRange(), NSRange(location: expected.utf16.count, length: 0))
            let caret = editor.firstRect(forCharacterRange: editor.selectedRange(), actualRange: nil)
            let fieldRect = window.convertToScreen(field.convert(field.bounds, to: nil))
            XCTAssertGreaterThanOrEqual(caret.minX, fieldRect.minX - caretTolerance)
            XCTAssertLessThanOrEqual(caret.minX, fieldRect.maxX + caretTolerance)
        }
        XCTAssertGreaterThan(editor.visibleRect.minX, 0, "The test must exercise horizontal scrolling")
    }

    private func summaryField(in view: NSView) -> NSTextField? {
        if let field = view as? NSTextField {
            return field
        }
        return view.subviews.lazy.compactMap { self.summaryField(in: $0) }.first
    }

    @Observable
    fileprivate final class Draft {
        var summary = ""
    }
}
