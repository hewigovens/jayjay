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

        let items = view.menuProvider(selection: DiffGutterSelection(lineRange: 1 ... 3, changedLineCount: 2))

        XCTAssertEqual(items.last?.title, "Abandon Selected Lines")
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

        let items = view.menuProvider(selection: DiffGutterSelection(lineRange: 2 ... 2, changedLineCount: 1))

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

        let items = view.menuProvider(selection: DiffGutterSelection(lineRange: 2 ... 3, changedLineCount: 2))

        XCTAssertEqual(items.last?.title, "Abandon Change Group")
    }

    func test_expandedChangedRange_returnsNilForEmptyDiff() {
        XCTAssertNil(DiffGutterGrouping.expandedChangedRange(in: [], containing: 1 ... 1))
    }

    private func line(_ style: DiffSpanStyle) -> DiffLine {
        DiffLine(
            oldLineNo: nil,
            newLineNo: nil,
            style: style,
            spans: [],
            noEofNewline: false
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

    func didSelectLines(_ lineRange: ClosedRange<Int>) {}
    func openDiffEdit() {}
    func abandonSelectedLines(in lineRange: ClosedRange<Int>) {}
}
