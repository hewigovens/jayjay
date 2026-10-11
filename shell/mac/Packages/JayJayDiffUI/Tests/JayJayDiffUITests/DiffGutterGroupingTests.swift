import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class DiffGutterGroupingTests: XCTestCase {
    func test_expandedChangedRange_groupsAdjacentAddedAndRemovedLines() {
        let lines = [
            line(.context),
            line(.removed),
            line(.removed),
            line(.added),
            line(.context)
        ]

        XCTAssertEqual(
            DiffGutterGrouping.expandedChangedRange(in: lines, containing: 3 ... 3),
            2 ... 4
        )
    }

    func test_expandedChangedRange_returnsNilForContextSelection() {
        let lines = [
            line(.context),
            line(.removed),
            line(.added),
            line(.context)
        ]

        XCTAssertNil(
            DiffGutterGrouping.expandedChangedRange(in: lines, containing: 1 ... 1)
        )
    }

    func test_expandedChangedRange_usesChangedLineWhenSelectionStartsOnContext() {
        let lines = [
            line(.context),
            line(.removed),
            line(.removed),
            line(.added),
            line(.context)
        ]

        XCTAssertEqual(
            DiffGutterGrouping.expandedChangedRange(in: lines, containing: 1 ... 3),
            2 ... 4
        )
    }

    func test_menuLabelsPartialContextSelectionAsSelectedLines() {
        let diff = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [
                line(.context),
                line(.removed),
                line(.removed),
                line(.added),
                line(.context)
            ],
            whitespaceOnlyHidden: false
        )
        let view = NativeDiffView(
            diff: diff,
            gutterActions: TestLineAbandoningActions()
        )

        let items = view.menuProvider(
            selection: DiffGutterSelection(lineRange: 1 ... 3, menuLineNumber: 1, changedLineCount: 2),
            displayLines: diffDisplayLines(lines: diff.lines),
            changeGroupsByIndex: [:]
        )

        XCTAssertEqual(items.last?.title, "Abandon Selected Lines")
    }

    func test_menuOffersEditingTheFileAtTheClickedNewLineOnly() {
        let diff = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [
                line(.context, old: 7, new: 7),
                line(.removed, old: 8, new: nil)
            ],
            whitespaceOnlyHidden: false
        )
        let view = NativeDiffView(diff: diff, gutterActions: TestLineAbandoningActions())
        let titles = { (lineNumber: Int) in
            view.menuProvider(
                selection: DiffGutterSelection(lineRange: lineNumber ... lineNumber, menuLineNumber: lineNumber, changedLineCount: 0),
                displayLines: diffDisplayLines(lines: diff.lines),
                changeGroupsByIndex: [:]
            ).map(\.title)
        }

        XCTAssertTrue(titles(1).contains("Edit File at Line 7"))
        XCTAssertTrue(titles(1).contains("Open in Zed at Line 7"))
        XCTAssertFalse(titles(2).contains { $0.hasPrefix("Edit File") || $0.hasPrefix("Open in") })
    }

    func test_menuLabelsSingleLineChangeAsSelectedLines() {
        let diff = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [
                line(.context),
                line(.removed),
                line(.context)
            ],
            whitespaceOnlyHidden: false
        )
        let view = NativeDiffView(
            diff: diff,
            gutterActions: TestLineAbandoningActions()
        )

        let items = view.menuProvider(
            selection: DiffGutterSelection(lineRange: 2 ... 2, menuLineNumber: 2, changedLineCount: 1),
            displayLines: diffDisplayLines(lines: diff.lines),
            changeGroupsByIndex: [:]
        )

        XCTAssertEqual(items.last?.title, "Abandon Selected Lines")
    }

    func test_menuLabelsMultiLineWholeChangeAsChangeGroup() {
        let diff = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [
                line(.context),
                line(.removed),
                line(.added),
                line(.context)
            ],
            whitespaceOnlyHidden: false
        )
        let view = NativeDiffView(
            diff: diff,
            gutterActions: TestLineAbandoningActions()
        )

        let items = view.menuProvider(
            selection: DiffGutterSelection(lineRange: 2 ... 3, menuLineNumber: 2, changedLineCount: 2),
            displayLines: diffDisplayLines(lines: diff.lines),
            changeGroupsByIndex: [:]
        )

        XCTAssertEqual(items.last?.title, "Abandon Change Group")
    }

    func test_expandedChangedRange_returnsNilForEmptyDiff() {
        XCTAssertNil(DiffGutterGrouping.expandedChangedRange(in: [], containing: 1 ... 1))
    }

    private func line(_ style: DiffSpanStyle, old: UInt32? = nil, new: UInt32? = nil) -> DiffLine {
        DiffLine(
            oldLineNo: old,
            newLineNo: new,
            style: style,
            spans: [],
            conflictKind: .none,
            noEofNewline: false,
            contextRegion: nil
        )
    }
}

private struct TestLineAbandoningActions: DiffGutterEditActions {
    var currentSelectedLineRange: ClosedRange<Int>? {
        nil
    }

    var canOpenDiffEdit: Bool {
        false
    }

    var canAbandonSelectedLines: Bool {
        true
    }

    var canEditFile: Bool {
        true
    }

    var externalEditorName: String? {
        "Zed"
    }

    func didSelectLines(_ lineRange: ClosedRange<Int>) {}
    func openDiffEdit() {}
    func abandonSelectedLines(in lineRange: ClosedRange<Int>) {}
    func editFile(atLine line: UInt32) {}
    func openInExternalEditor(atLine line: UInt32) {}
}
