@testable import JayJay
import XCTest

final class ConflictEditorGateTests: XCTestCase {
    func testSupportedConflictInMutableChangeCanBeEdited() {
        XCTAssertTrue(
            ChangeDetailView.canEnterConflictEditor(
                info: mockChangeInfo(),
                hunk: testHunk(supportsConflictEditor: true),
                isCompareMode: false
            )
        )
    }

    func testImmutableAndComparedConflictsCannotBeEdited() {
        let hunk = testHunk(supportsConflictEditor: true)
        XCTAssertFalse(
            ChangeDetailView.canEnterConflictEditor(
                info: mockChangeInfo(isImmutable: true),
                hunk: hunk,
                isCompareMode: false
            )
        )
        XCTAssertFalse(
            ChangeDetailView.canEnterConflictEditor(
                info: mockChangeInfo(),
                hunk: hunk,
                isCompareMode: true
            )
        )
    }
}
