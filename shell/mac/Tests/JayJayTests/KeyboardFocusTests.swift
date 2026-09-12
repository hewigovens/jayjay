import AppKit
@testable import JayJay
import SwiftUI
import XCTest

@MainActor
final class KeyboardFocusTests: XCTestCase {
    func testTabCyclesRegisteredStopsAndWraps() {
        let focus = KeyboardFocus()
        var activated: [KeyboardFocusStop] = []
        focus.register(.fileList, token: UUID()) { activated.append(.fileList) }
        focus.register(.refresh, token: UUID()) { activated.append(.refresh) }
        focus.register(.filterToggle, token: UUID()) { activated.append(.filterToggle) }

        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.tab)))
        XCTAssertEqual(activated, [.fileList])
        XCTAssertNil(focus.control)
        focus.activePane = .fileColumn
        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.tab)))
        XCTAssertEqual(focus.control, .filterToggle)
        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.tab)))
        XCTAssertEqual(focus.control, .refresh)
        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.space)))
        XCTAssertEqual(activated, [.fileList, .refresh])
        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.tab)))
        XCTAssertEqual(focus.activePane, .dag)
        XCTAssertNil(focus.control)
        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.tab, modifiers: .shift)))
        XCTAssertEqual(focus.control, .refresh)
    }

    func testPaneChangesAndUnregisteringClearTheFocusedControl() {
        let focus = KeyboardFocus()
        let token = UUID()
        focus.register(.settings, token: token) {}
        focus.handleKey(Self.key(KeyCode.tab))
        XCTAssertEqual(focus.control, .settings)
        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.escape)))
        XCTAssertNil(focus.control)
        XCTAssertFalse(focus.handleKey(Self.key(KeyCode.escape)))

        focus.handleKey(Self.key(KeyCode.tab))
        focus.activePane = .fileColumn
        XCTAssertNil(focus.control)

        focus.handleKey(Self.key(KeyCode.tab))
        focus.unregister(.settings, token: UUID())
        XCTAssertEqual(focus.control, .settings, "a stale token must not remove a live registration")
        focus.unregister(.settings, token: token)
        XCTAssertNil(focus.control)
    }

    func testRegisteredViewUsesUpdatedInputs() {
        let focus = KeyboardFocus()
        let model = RegistrationModel()
        let host = NSHostingView(rootView: RegistrationView(model: model).environment(focus))
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 200, height: 100),
            styleMask: [.borderless], backing: .buffered, defer: false
        )
        window.contentView = host
        host.layoutSubtreeIfNeeded()
        RunLoop.main.run(until: Date().addingTimeInterval(0.1))
        model.paths = ["new.txt"]
        model.selected = "new.txt"
        host.layoutSubtreeIfNeeded()
        RunLoop.main.run(until: Date().addingTimeInterval(0.1))

        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.tab)))
        XCTAssertEqual(model.activatedPaths, ["new.txt"])
        XCTAssertEqual(model.selected, "new.txt")
    }

    func testTabFocusesInputsWithoutConsumingTheirTextCommands() {
        let focus = KeyboardFocus()
        var focused: [KeyboardFocusStop] = []
        for stop in [KeyboardFocusStop.revsetInput, .commitSummary, .commitDescription] {
            focus.register(stop, token: UUID()) { focused.append(stop) }
        }
        for stop in [KeyboardFocusStop.revsetInput, .commitSummary, .commitDescription] {
            XCTAssertTrue(focus.handleKey(Self.key(KeyCode.tab)))
            XCTAssertEqual(focus.control, stop)
            XCTAssertEqual(focused.last, stop)
            XCTAssertFalse(focus.handleKey(Self.key(KeyCode.space)))
            XCTAssertFalse(focus.handleKey(Self.key(KeyCode.returnKey)))
        }
        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.tab, modifiers: .shift)))
        XCTAssertEqual(focus.control, .commitSummary)
        focus.updateInputFocus(.commitDescription, isFocused: false)
        XCTAssertEqual(focus.control, .commitSummary)
    }

    func testDiffEditSuspendsTraversalAndControlActivation() {
        let focus = KeyboardFocus()
        var activated = false
        focus.register(.refresh, token: UUID()) { activated = true }
        focus.handleKey(Self.key(KeyCode.tab))
        focus.isSuspended = true
        focus.updateInputFocus(.commitSummary, isFocused: true)
        XCTAssertNil(focus.control)
        XCTAssertFalse(focus.handleKey(Self.key(KeyCode.tab)))
        XCTAssertFalse(focus.handleKey(Self.key(KeyCode.space)))
        XCTAssertFalse(activated)
        focus.isSuspended = false
        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.tab)))
        XCTAssertTrue(focus.handleKey(Self.key(KeyCode.space)))
        XCTAssertTrue(activated)
    }

    private static func key(_ keyCode: UInt16, modifiers: NSEvent.ModifierFlags = []) -> NSEvent {
        NSEvent.keyEvent(
            with: .keyDown, location: .zero, modifierFlags: modifiers, timestamp: 0, windowNumber: 0,
            context: nil, characters: "", charactersIgnoringModifiers: "", isARepeat: false, keyCode: keyCode
        )!
    }
}

@MainActor @Observable
private final class RegistrationModel {
    var paths = ["old.txt"]
    var selected = "old.txt"
    var activatedPaths: [String] = []
}

private struct RegistrationView: View {
    let model: RegistrationModel

    var body: some View {
        let paths = model.paths
        Text(paths.joined(separator: ","))
            .keyboardFocusStop(.fileList) {
                model.activatedPaths = paths
                if !paths.contains(model.selected), let first = paths.first {
                    model.selected = first
                }
            }
    }
}
