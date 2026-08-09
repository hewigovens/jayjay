@testable import JayJay
import JayJayCore
import XCTest

final class DiffEditGateTests: XCTestCase {
    func testMutableChangeCanEnterDiffEdit() {
        XCTAssertTrue(ChangeDetailView.canEnterDiffEdit(info: mockChangeInfo(), isCompareMode: false))
    }

    func testImmutableChangeCannotEnterDiffEdit() {
        XCTAssertFalse(ChangeDetailView.canEnterDiffEdit(info: mockChangeInfo(isImmutable: true), isCompareMode: false))
    }

    func testConflictedChangeCannotEnterDiffEdit() {
        XCTAssertFalse(ChangeDetailView.canEnterDiffEdit(info: mockChangeInfo(hasConflict: true), isCompareMode: false))
    }

    func testEmptyChangeCannotEnterDiffEdit() {
        XCTAssertFalse(ChangeDetailView.canEnterDiffEdit(info: mockChangeInfo(isEmpty: true), isCompareMode: false))
    }

    func testCompareModeCannotEnterDiffEdit() {
        XCTAssertFalse(ChangeDetailView.canEnterDiffEdit(info: mockChangeInfo(), isCompareMode: true))
    }
}
