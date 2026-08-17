@testable import JayJay
import JayJayCore
import XCTest

final class DiffEditGateTests: XCTestCase {
    func testMutableChangeCanEnterDiffEdit() {
        XCTAssertTrue(ChangeDetailView.canEnterDiffEdit(info: change(), isCompareMode: false))
    }

    func testImmutableChangeCannotEnterDiffEdit() {
        XCTAssertFalse(ChangeDetailView.canEnterDiffEdit(info: change(isImmutable: true), isCompareMode: false))
    }

    func testConflictedChangeCannotEnterDiffEdit() {
        XCTAssertFalse(ChangeDetailView.canEnterDiffEdit(info: change(hasConflict: true), isCompareMode: false))
    }

    func testEmptyChangeCannotEnterDiffEdit() {
        XCTAssertFalse(ChangeDetailView.canEnterDiffEdit(info: change(isEmpty: true), isCompareMode: false))
    }

    func testCompareModeCannotEnterDiffEdit() {
        XCTAssertFalse(ChangeDetailView.canEnterDiffEdit(info: change(), isCompareMode: true))
    }

    private func change(hasConflict: Bool = false, isEmpty: Bool = false, isImmutable: Bool = false) -> ChangeInfo {
        ChangeInfo(
            changeId: ShortId(id: "c-1", shortLen: 1),
            commitId: ShortId(id: "abc123", shortLen: 1),
            description: "change",
            author: .tester,
            parents: [],
            bookmarks: [],
            tags: [],
            workspaces: [],
            isWorkingCopy: false,
            hasConflict: hasConflict,
            isEmpty: isEmpty,
            isImmutable: isImmutable,
            isDivergent: false
        )
    }
}
