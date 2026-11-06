import AppKit
@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class ShortcutStoreTests: XCTestCase {
    private var defaults: UserDefaults!
    private var suite: String!

    override func setUp() {
        super.setUp()
        suite = "ShortcutStoreTests.\(UUID().uuidString)"
        defaults = UserDefaults(suiteName: suite)
    }

    override func tearDown() {
        defaults.removePersistentDomain(forName: suite)
        super.tearDown()
    }

    func testRebindPersistsAndResetPreservesUnknownIDs() throws {
        defaults.set(["future.action": ["future-chord"], "repo.fetch": ["not-valid"]], forKey: ShortcutStore.storageKey)
        let store = ShortcutStore(defaults: defaults)
        try store.apply(.assign(chord: "alt-cmd-r"), to: "repo.refresh")
        let binding = try XCTUnwrap(store.byID["repo.refresh"]?.primary?.keyboardShortcut)
        XCTAssertEqual(binding.key, "r")
        XCTAssertEqual(binding.modifiers, [.command, .option])
        let reopened = ShortcutStore(defaults: defaults)
        XCTAssertEqual(reopened.byID["repo.refresh"]?.primary?.text, "alt-cmd-r")
        XCTAssertNil(reopened.stored["repo.fetch"])
        let data = Data(#"[{"id":"a-help-topic","title":"Help","summary":"Help","category":"Help","keywords":[],"shortcut":"legacy hint","command":"repo.refresh","guideAnchor":"help"}]"#.utf8)
        let feature = try XCTUnwrap(HelpFeatureIndex.decode(data: data).first)
        XCTAssertEqual(feature.effectiveShortcut(in: reopened), reopened.byID["repo.refresh"]?.primary?.caps.joined())
        try reopened.apply(.unbind, to: "repo.refresh")
        XCTAssertNil(reopened.byID["repo.refresh"]?.primary)
        XCTAssertNil(feature.effectiveShortcut(in: reopened))
        XCTAssertFalse(ShortcutGuide.columns(bindings: reopened.bindings).flatMap { $0 }.flatMap(\.entries).contains { $0.id == "repo.refresh" })
        try reopened.apply(.reset, to: "repo.refresh")
        XCTAssertEqual(reopened.byID["repo.refresh"]?.primary?.text, "cmd-r")
        try reopened.apply(.resetAll)
        XCTAssertEqual(reopened.stored, ["future.action": ["future-chord"]])
        XCTAssertEqual(ShortcutStore(defaults: defaults).byID["repo.refresh"]?.primary?.text, "cmd-r")
    }

    func testConflictRequiresReplacementAndRemovesOldMenuEquivalent() throws {
        let store = ShortcutStore(defaults: defaults)
        let recorder = ShortcutRecorder(store: store)
        recorder.apply(.assign(chord: "cmd-l"), id: "repo.refresh")
        XCTAssertNotNil(recorder.message)
        XCTAssertTrue(recorder.canReplace)
        XCTAssertEqual(recorder.pendingChord?.text, "cmd-l")
        XCTAssertEqual(store.byID["repo.refresh"]?.primary?.text, "cmd-r")
        XCTAssertNotNil(store.byID["repo.filter_by_revset"]?.primary?.keyboardShortcut)
        recorder.replace()
        XCTAssertEqual(store.byID["repo.refresh"]?.primary?.text, "cmd-l")
        XCTAssertNil(store.byID["repo.filter_by_revset"]?.primary?.keyboardShortcut)
        XCTAssertEqual(store.stored["repo.filter_by_revset"], [])
        let stored = store.stored
        for id in ["app.quit", "future.action"] {
            recorder.apply(.unbind, id: id)
            XCTAssertNil(recorder.message)
            XCTAssertNil(recorder.commandID)
            XCTAssertEqual(store.stored, stored)
        }
    }

    func testRecorderConsumesCommandBeforeDispatch() throws {
        let store = ShortcutStore(defaults: defaults)
        let recorder = ShortcutRecorder(store: store)
        let window = RecorderWindow(contentRect: NSRect(x: 0, y: 0, width: 100, height: 100), styleMask: [.borderless], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer { recorder.cancel(); window.close() }
        let view = RecorderView()
        window.contentView = view
        recorder.begin(id: "repo.refresh", view: view)
        let flags = try XCTUnwrap(NSEvent.keyEvent(with: .flagsChanged, location: .zero, modifierFlags: [.shift, .command], timestamp: 0,
                                                  windowNumber: window.windowNumber, context: nil, characters: "", charactersIgnoringModifiers: "", isARepeat: false, keyCode: 56))
        XCTAssertNil(recorder.capture(flags))
        XCTAssertEqual(recorder.heldCaps, ["⇧", "⌘"])
        let event = key(15, "r", modifiers: .command)
        XCTAssertNil(recorder.capture(event))
        XCTAssertFalse(recorder.isRecording)
        XCTAssertEqual(store.byID["repo.refresh"]?.primary?.text, "cmd-r")
    }

    func testRecorderYieldsAfterSearchTakesFocusWithoutUnbinding() throws {
        let store = ShortcutStore(defaults: defaults)
        let recorder = ShortcutRecorder(store: store)
        let window = RecorderWindow(contentRect: NSRect(x: 0, y: 0, width: 100, height: 100), styleMask: [.borderless], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer { recorder.cancel(); window.close() }
        let view = RecorderView()
        let search = NSTextView()
        window.contentView = view
        view.addSubview(search)
        recorder.begin(id: "repo.refresh", view: view)
        XCTAssertTrue(window.makeFirstResponder(search))
        XCTAssertNotNil(recorder.capture(key(51, "\u{7F}")))
        XCTAssertEqual(store.byID["repo.refresh"]?.primary?.text, "cmd-r")
        XCTAssertTrue(store.stored.isEmpty)
        XCTAssertFalse(recorder.isRecording)
    }

    func testRecorderCancelsWhenRowDisappearsAndEscapeDismissesConflicts() throws {
        let store = ShortcutStore(defaults: defaults)
        let recorder = ShortcutRecorder(store: store)
        let window = RecorderWindow(contentRect: NSRect(x: 0, y: 0, width: 100, height: 100), styleMask: [.borderless], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer {
            recorder.cancel()
            window.close()
        }
        let view = RecorderView()
        window.contentView = NSView()
        window.contentView?.addSubview(view)
        recorder.begin(id: "repo.refresh", view: view)
        view.removeFromSuperview()
        XCTAssertNotNil(recorder.capture(key(15, "r", modifiers: .command)))
        XCTAssertFalse(recorder.isRecording)
        XCTAssertNil(recorder.commandID)
        XCTAssertTrue(store.stored.isEmpty)

        window.contentView?.addSubview(view)
        for chord in ["cmd-l", "ctrl-n"] {
            recorder.begin(id: "repo.refresh", view: view)
            recorder.apply(.assign(chord: chord), id: "repo.refresh")
            XCTAssertNotNil(recorder.pending)
            XCTAssertEqual(recorder.pendingChord?.text, chord)
            XCTAssertNotNil(recorder.capture(key(15, "r", modifiers: .command)))
            XCTAssertNotNil(recorder.pending)
            XCTAssertNil(recorder.capture(key(KeyCode.escape, "\u{1B}")))
            XCTAssertNil(recorder.pending)
            XCTAssertNil(recorder.commandID)
            XCTAssertTrue(store.stored.isEmpty)
        }

        try store.apply(.assign(chord: "alt-cmd-r"), to: "repo.refresh")
        try store.apply(.assign(chord: "cmd-r"), to: "repo.filter_by_revset")
        recorder.begin(id: "repo.refresh", view: view)
        recorder.apply(.reset, id: "repo.refresh")
        XCTAssertEqual(recorder.pendingChord?.text, "cmd-r")
        recorder.replace()
        XCTAssertEqual(store.byID["repo.refresh"]?.primary?.text, "cmd-r")
        XCTAssertNil(store.byID["repo.filter_by_revset"]?.primary)
    }

    private func key(_ code: UInt16, _ characters: String, modifiers: NSEvent.ModifierFlags = []) -> NSEvent {
        NSEvent.keyEvent(with: .keyDown, location: .zero, modifierFlags: modifiers, timestamp: 0, windowNumber: 0,
                        context: nil, characters: characters, charactersIgnoringModifiers: characters, isARepeat: false, keyCode: code)!
    }

    private final class RecorderWindow: NSWindow {
        override var isKeyWindow: Bool { true }
    }

    private final class RecorderView: NSView {
        override var acceptsFirstResponder: Bool { true }
    }
}
