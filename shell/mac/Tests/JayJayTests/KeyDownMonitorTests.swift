import AppKit
@testable import JayJay
import XCTest

@MainActor
final class KeyDownMonitorTests: XCTestCase {
    func testKeysOnlyAppKitWouldBeepForAreConsumed() {
        let window = NSWindow(contentRect: .zero, styleMask: [.borderless], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer { window.close() }

        let beeping = [
            key(7, "x"),
            key(26, "7"),
            key(41, ";"),
            key(KeyCode.space, " "),
            key(KeyCode.delete),
            key(KeyCode.forwardDelete),
            key(KeyCode.escape, "\u{1B}")
        ]
        for event in beeping {
            XCTAssertTrue(consumes(event), "key \(event.keyCode) reached AppKit's responder chain")
        }
        XCTAssertTrue(consumes(key(7, "x", modifiers: .shift)), "Shift must not hand the key back to AppKit")
        XCTAssertTrue(consumes(key(7, "x"), firstResponder: window), "a key window with no key view still belongs to the pane")
        XCTAssertTrue(
            consumes(key(7, "x"), firstResponder: NSTableView(frame: .zero)),
            "a List's table view does not keep keys the pane's own navigation replaces"
        )
    }

    func testKeysOwnedByAnotherResponderArePassedThrough() {
        XCTAssertFalse(consumes(key(7, "x", modifiers: .command)))
        XCTAssertFalse(consumes(key(35, "p", modifiers: .control)))
        XCTAssertFalse(consumes(key(123, "\u{F702}", modifiers: .option)))
        let responderOwned = [
            key(KeyCode.tab, "\t"),
            key(KeyCode.returnKey, "\r"),
            key(KeyCode.keypadEnter, "\u{3}"),
            key(KeyCode.upArrow, "\u{F700}"),
            key(KeyCode.downArrow, "\u{F701}"),
            key(121, "\u{F72D}"),
            key(122, "\u{F704}")
        ]
        for event in responderOwned {
            XCTAssertFalse(consumes(event), "key \(event.keyCode) keeps its AppKit meaning")
        }
        XCTAssertFalse(consumes(key(7, "x"), firstResponder: NSTextView()), "a focused text view keeps typing")
        XCTAssertFalse(consumes(key(KeyCode.space, " "), firstResponder: NSButton()), "a focused control keeps Space")
    }

    private func consumes(_ event: NSEvent, firstResponder: NSResponder? = nil) -> Bool {
        KeyDownMonitor.Coordinator.consumesUnhandledKey(event, firstResponder: firstResponder)
    }

    private func key(
        _ keyCode: UInt16,
        _ characters: String = "",
        modifiers: NSEvent.ModifierFlags = []
    ) -> NSEvent {
        NSEvent.keyEvent(
            with: .keyDown, location: .zero, modifierFlags: modifiers, timestamp: 0, windowNumber: 0,
            context: nil, characters: characters, charactersIgnoringModifiers: characters, isARepeat: false,
            keyCode: keyCode
        )!
    }
}
