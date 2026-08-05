import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class NativeDiffViewReviewNoteTests: XCTestCase {
    func testReviewNoteMenuUsesClickedChangedLineAsAnchor() throws {
        let diff = FileDiff(
            path: "file.txt",
            language: "",
            lines: [
                line(old: 1, new: 1, style: .context, text: "before"),
                line(old: nil, new: 2, style: .added, text: "first"),
                line(old: nil, new: 3, style: .added, text: "second"),
                line(old: 2, new: 4, style: .context, text: "after")
            ],
            whitespaceOnlyHidden: false
        )
        let noteActions = CapturingNoteActions()
        let view = NativeDiffView(diff: diff, gutterActions: noteActions)
        let displayLines = diffDisplayLines(lines: diff.lines)
        let groups = Dictionary(uniqueKeysWithValues: changeGroups(lines: displayLines).map { ($0.index, $0) })

        let items = view.menuProvider(
            selection: DiffGutterSelection(lineRange: 2 ... 3, menuLineNumber: 3, changedLineCount: 2),
            changeGroupsByIndex: groups
        )
        let add = try XCTUnwrap(items.first { $0.title == "Add Review Note" })
        add.action?()

        let anchor = try XCTUnwrap(noteActions.addedAnchor)
        XCTAssertEqual(anchor.line, 3)
        XCTAssertEqual(anchor.displayLine, 3)
        XCTAssertEqual(anchor.excerpt, "second")
        XCTAssertEqual(anchor.side, .new)
    }

    func testReviewNoteMenuHidesAddWhenActiveNoteExists() throws {
        let diff = FileDiff(
            path: "file.txt",
            language: "",
            lines: [
                line(old: 1, new: 1, style: .context, text: "before"),
                line(old: nil, new: 2, style: .added, text: "first"),
                line(old: nil, new: 3, style: .added, text: "second"),
                line(old: 2, new: 4, style: .context, text: "after")
            ],
            whitespaceOnlyHidden: false
        )
        let displayLines = diffDisplayLines(lines: diff.lines)
        let groups = Dictionary(uniqueKeysWithValues: changeGroups(lines: displayLines).map { ($0.index, $0) })
        let anchorView = NativeDiffView(diff: diff)
        let anchors = anchorView.noteAnchorsByLineNumber(lines: displayLines, groups: groups)
        let anchor = try XCTUnwrap(anchors[3])
        let noteActions = CapturingNoteActions(notes: [anchor: [
            DiffReviewNoteSummary(id: "n1", body: "First note")
        ]])
        let view = NativeDiffView(diff: diff, gutterActions: noteActions)

        let items = view.menuProvider(
            selection: DiffGutterSelection(lineRange: 2 ... 3, menuLineNumber: 3, changedLineCount: 2),
            changeGroupsByIndex: groups
        )
        XCTAssertNil(items.first { $0.title == "Add Review Note" })
        XCTAssertNotNil(items.first { $0.title == "Edit Review Note" })
        let resolve = try XCTUnwrap(items.first { $0.title == "Resolve Review Note" })
        resolve.action?()

        XCTAssertEqual(noteActions.resolvedNoteId, "n1")
    }

    func testNoteMarkersMapSummariesToTheirAnchoredDisplayLine() {
        let diff = FileDiff(
            path: "file.txt",
            language: "",
            lines: [
                line(old: 1, new: 1, style: .context, text: "before"),
                line(old: nil, new: 2, style: .added, text: "first"),
                line(old: nil, new: 3, style: .added, text: "second"),
                line(old: 2, new: 4, style: .context, text: "after")
            ],
            whitespaceOnlyHidden: false
        )
        let displayLines = diffDisplayLines(lines: diff.lines)
        let view = NativeDiffView(diff: diff)

        let byLine = view.noteSummariesByDisplayLine(
            displayLines: displayLines,
            groups: changeGroups(lines: displayLines),
            notes: [
                DiffReviewNoteSummary(id: "n1", body: "on second", side: .new, line: 3),
                DiffReviewNoteSummary(id: "n2", body: "elsewhere", side: .new, line: 9),
                DiffReviewNoteSummary(id: "n3", body: "wrong side", side: .old, line: 3)
            ]
        )

        XCTAssertEqual(byLine.keys.sorted(), [3])
        XCTAssertEqual(byLine[3]?.map(\.id), ["n1"])
    }

    func testRenderRowsSpliceNoteBodyBelowAnchorWithoutRenumbering() {
        let displayLines = [
            line(old: 1, new: 1, style: .context, text: "before"),
            line(old: nil, new: 2, style: .added, text: "    first"),
            line(old: 2, new: 3, style: .context, text: "after")
        ]

        let rows = diffRenderRows(displayLines: displayLines, notesByLine: [
            2: [DiffReviewNoteSummary(id: "n1", body: "check this\nand this", side: .new, line: 2)]
        ])

        XCTAssertEqual(rows.count, 5)
        guard case let .line(_, anchorNumber) = rows[1],
              case let .note(firstText, firstIndent, firstIsFirst, firstIsLast) = rows[2],
              case let .note(secondText, _, secondIsFirst, secondIsLast) = rows[3],
              case let .line(_, afterNumber) = rows[4]
        else { return XCTFail("unexpected row order: \(rows)") }
        XCTAssertEqual(anchorNumber, 2)
        XCTAssertEqual(firstText, "check this")
        XCTAssertEqual(firstIndent, "    ", "bubble rows inherit the anchor line's leading whitespace")
        XCTAssertTrue(firstIsFirst)
        XCTAssertFalse(firstIsLast)
        XCTAssertEqual(secondText, "and this")
        XCTAssertFalse(secondIsFirst)
        XCTAssertTrue(secondIsLast, "the bubble must close on the note's final body line")
        XCTAssertEqual(afterNumber, 3, "diff rows keep their unspliced display line numbers")
    }

    func testRenderRowsSkipStaleAndResolvedNotes() {
        let displayLines = [
            line(old: nil, new: 1, style: .added, text: "changed"),
            line(old: 1, new: 2, style: .context, text: "after")
        ]

        let rows = diffRenderRows(displayLines: displayLines, notesByLine: [
            1: [
                DiffReviewNoteSummary(id: "n1", body: "old anchor", side: .new, line: 1, isStale: true),
                DiffReviewNoteSummary(id: "n2", body: "done", side: .new, line: 1, isResolved: true)
            ]
        ])

        XCTAssertEqual(rows.count, 2, "stale and resolved notes keep their marker but must not expand into the diff")
    }

    func testRenderRowsWithoutNotesMirrorDisplayLines() {
        let displayLines = [
            line(old: 1, new: 1, style: .context, text: "a"),
            line(old: nil, new: 2, style: .added, text: "b")
        ]

        let rows = diffRenderRows(displayLines: displayLines, notesByLine: [:])

        XCTAssertEqual(rows.count, 2)
        for (index, row) in rows.enumerated() {
            guard case let .line(_, number) = row else { return XCTFail("unexpected note row") }
            XCTAssertEqual(number, index + 1)
        }
    }

    private func line(
        old: UInt32?,
        new: UInt32?,
        style: DiffSpanStyle,
        text: String
    ) -> DiffLine {
        DiffLine(
            oldLineNo: old,
            newLineNo: new,
            style: style,
            spans: [DiffSpan(text: text, style: style, token: .plain)],
            conflictKind: .none,
            noEofNewline: false,
            contextRegion: nil
        )
    }
}

private final class CapturingNoteActions: DiffGutterNoteActions {
    var currentSelectedLineRange: ClosedRange<Int>?
    var reviewNotesEnabled = true
    var addedAnchor: DiffReviewNoteAnchor?
    var notes: [DiffReviewNoteAnchor: [DiffReviewNoteSummary]]
    var resolvedNoteId: String?

    init(notes: [DiffReviewNoteAnchor: [DiffReviewNoteSummary]] = [:]) {
        self.notes = notes
    }

    func didSelectLines(_ lineRange: ClosedRange<Int>) {
        currentSelectedLineRange = lineRange
    }

    func activeNotes(anchor: DiffReviewNoteAnchor) -> [DiffReviewNoteSummary] {
        notes[anchor] ?? []
    }

    func addNote(anchor: DiffReviewNoteAnchor) {
        addedAnchor = anchor
    }

    func editNote(id: String) {}
    func deleteNote(id: String) {}
    func resolveNote(id: String) {
        resolvedNoteId = id
    }
}
