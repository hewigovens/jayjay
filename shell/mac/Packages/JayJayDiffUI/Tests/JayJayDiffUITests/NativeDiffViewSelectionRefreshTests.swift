import AppKit
import JayJayCore
@testable import JayJayDiffUI
import SwiftUI
import XCTest

@MainActor
final class NativeDiffViewSelectionRefreshTests: XCTestCase {
    func testSelectionChangeRefreshesOnlyTheGutter() {
        let diff = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [addedLine(1), addedLine(2)],
            whitespaceOnlyHidden: false
        )
        let initialActions = SelectionActions(selected: [1, 2])
        let hosting = NSHostingView(rootView: view(diff: diff, actions: initialActions))
        let window = host(hosting)
        waitForRender(hosting)
        guard let container = findContainer(in: hosting) else {
            return XCTFail("DiffTextContainerView not found in hierarchy")
        }
        let contentEdited = expectation(description: "Selection refresh edited diff content")
        contentEdited.isInverted = true
        let observer = NotificationCenter.default.addObserver(
            forName: NSTextStorage.didProcessEditingNotification,
            object: container.textView.textStorage,
            queue: .main
        ) { _ in
            contentEdited.fulfill()
        }
        defer { NotificationCenter.default.removeObserver(observer) }
        XCTAssertEqual(glyphCount("✓", in: container.gutterTextView.string), 2)

        let updatedActions = SelectionActions(selected: [2])
        hosting.rootView = view(diff: diff, actions: updatedActions)
        waitForRender(hosting)

        wait(for: [contentEdited], timeout: 0.05)
        XCTAssertEqual(glyphCount("✓", in: container.gutterTextView.string), 1)
        XCTAssertEqual(glyphCount("□", in: container.gutterTextView.string), 1)
        container.gutterTextView.toggleLineCheckbox?(1)
        XCTAssertEqual(updatedActions.toggledLines, [1])
        XCTAssertTrue(initialActions.toggledLines.isEmpty)
        _ = window
    }

    private func view(diff: FileDiff, actions: SelectionActions) -> some View {
        NativeDiffView(diff: diff, gutterActions: actions, contentGeneration: 1)
            .frame(width: 366, height: 160)
    }

    private func addedLine(_ number: Int) -> DiffLine {
        DiffLine(
            oldLineNo: nil,
            newLineNo: UInt32(number),
            style: .added,
            spans: [DiffSpan(text: "line \(number)", style: .added, token: .plain)],
            conflictKind: .none,
            noEofNewline: false,
            contextRegion: nil
        )
    }

    private func glyphCount(_ glyph: Character, in text: String) -> Int {
        text.reduce(into: 0) { count, character in
            if character == glyph {
                count += 1
            }
        }
    }

    private func host(_ hosting: NSView) -> NSWindow {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 366, height: 160),
            styleMask: [.titled],
            backing: .buffered,
            defer: false
        )
        window.contentView = hosting
        return window
    }

    private func waitForRender(_ hosting: NSView) {
        hosting.layoutSubtreeIfNeeded()
        for _ in 0 ..< 5 {
            RunLoop.main.run(until: Date().addingTimeInterval(0.05))
        }
    }

    private func findContainer(in view: NSView) -> DiffTextContainerView? {
        if let container = view as? DiffTextContainerView {
            return container
        }
        for subview in view.subviews {
            if let container = findContainer(in: subview) {
                return container
            }
        }
        return nil
    }
}

private final class SelectionActions: DiffGutterSelectionActions {
    let selected: Set<Int>
    private(set) var toggledLines: [Int] = []

    init(selected: Set<Int>) {
        self.selected = selected
    }

    var currentSelectedLineRange: ClosedRange<Int>? {
        nil
    }

    func didSelectLines(_ lineRange: ClosedRange<Int>) {}

    func selectFile() {}

    func selectChangeGroup(_ lineRange: ClosedRange<Int>) {}

    func lineCheckboxState(for lineNumber: Int) -> DiffGutterCheckboxState? {
        selected.contains(lineNumber) ? .selected : .unselected
    }

    func toggleLineCheckbox(_ lineNumber: Int) {
        toggledLines.append(lineNumber)
    }
}
