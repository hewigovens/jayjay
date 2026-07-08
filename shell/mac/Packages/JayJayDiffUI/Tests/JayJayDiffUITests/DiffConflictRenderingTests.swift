import AppKit
import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class DiffConflictRenderingTests: XCTestCase {
    func test_conflictLabelReplacesRawMarkerText() {
        let header = line("<<<<<<< conflict 1 of 1", kind: .start)

        XCTAssertEqual(conflictLabel(for: header), "Conflict 1 of 1")
    }

    func test_conflictLabelIgnoresContentLines() {
        let content = line("+line2 FEATURE", kind: .added)

        XCTAssertNil(conflictLabel(for: content))
    }

    func test_conflictHeaderUsesConflictPalette() {
        let theme = DiffColors(isDark: false)
        let header = line("<<<<<<< conflict 1 of 1", kind: .start)

        assertSameColor(theme.lineBg(header), theme.conflictHeaderBg)
        assertSameColor(theme.lineText(header), theme.conflictHeaderText)
    }

    /// Conflict-block structure is asserted in Rust (jj-diff builds_first_class_conflict_blocks); this covers the production display pipeline the gutter/SBS views actually call through FFI.
    func test_conflictDisplayLinesCollapseToProductionRendering() {
        let lines = [
            line("<<<<<<< conflict 1 of 1", kind: .start, newLineNo: 1),
            line("base", kind: .removed, style: .removed, newLineNo: 2),
            line("+++++++ side #1", kind: .section, newLineNo: 3),
            line("ours", kind: .added, newLineNo: 4),
            line(">>>>>>> conflict 1 of 1 ends", kind: .end, newLineNo: 5)
        ]

        let displayLines = diffDisplayLines(lines: lines)
        XCTAssertEqual(displayLines.count, 3)
        XCTAssertEqual(displayLines[0].rawText, "Conflict 1 of 1 · ◆ Side #1")
        XCTAssertEqual(displayLines[1].rawText, "base")
        XCTAssertEqual(displayLines[2].rawText, "◆ │ ours")
        XCTAssertEqual(wrapDiffLines(lines: displayLines, cols: 80).count, displayLines.count)
        XCTAssertEqual(buildSideBySideRows(lines: lines).count, displayLines.count)
    }

    private func line(
        _ text: String,
        kind: ConflictLineKind,
        style: DiffSpanStyle = .added,
        newLineNo: UInt32 = 1
    ) -> DiffLine {
        DiffLine(
            oldLineNo: nil,
            newLineNo: newLineNo,
            style: style,
            spans: [
                DiffSpan(text: text, style: style, token: .plain)
            ],
            conflictKind: kind,
            noEofNewline: false
        )
    }

}
