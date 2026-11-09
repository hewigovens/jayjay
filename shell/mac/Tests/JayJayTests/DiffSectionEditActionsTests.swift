@testable import JayJay
import JayJayCore
import XCTest

final class DiffSectionEditActionsTests: XCTestCase {
    func testSelectedGutterRowsMapToTheDisplayedLinesOfAModifiedBlock() {
        let diff = computeFileDiff(
            path: "notes.txt",
            oldContent: "one\ntwo\nthree\n",
            newContent: "ONE\nTWO\nthree\n",
            ignoreWhitespace: false
        )
        let displayLines = diffDisplayLines(lines: diff.lines)
        XCTAssertEqual(displayLines.prefix(4).map(\.style), [.removed, .removed, .added, .added])

        XCTAssertEqual(
            DiffSection.changedLineKeys(in: displayLines, lineRange: 1 ... 2),
            ["removed|1|-", "removed|2|-"]
        )
    }
}
