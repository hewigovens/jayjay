@testable import JayJay
import JayJayCore
import XCTest

/// Routing for diff-edit removeFromSource: the in-place @ row patch is only safe for a leaf working copy; any other rev rebases descendants and must take the full refresh.
final class DiffEditRoutingTests: XCTestCase {
    func testLeafWorkingCopyByChangeIdPatchesInPlace() {
        let changes = [change("wc", parents: ["p1"], isWorkingCopy: true), change("p1")]
        XCTAssertTrue(RepoViewModel.canPatchWorkingCopyRowInPlace(rev: "c-wc", changes: changes))
    }

    func testLeafWorkingCopyByCommitIdPatchesInPlace() {
        let changes = [change("wc", parents: ["p1"], isWorkingCopy: true), change("p1")]
        XCTAssertTrue(RepoViewModel.canPatchWorkingCopyRowInPlace(rev: "wc", changes: changes))
    }

    func testNonWorkingCopyRevNeedsFullRefresh() {
        let changes = [change("wc", parents: ["p1"], isWorkingCopy: true), change("p1")]
        XCTAssertFalse(RepoViewModel.canPatchWorkingCopyRowInPlace(rev: "c-p1", changes: changes))
    }

    func testWorkingCopyWithChildNeedsFullRefresh() {
        let changes = [change("child", parents: ["wc"]), change("wc", parents: ["p1"], isWorkingCopy: true), change("p1")]
        XCTAssertFalse(RepoViewModel.canPatchWorkingCopyRowInPlace(rev: "c-wc", changes: changes))
    }

    func testMissingWorkingCopyNeedsFullRefresh() {
        XCTAssertFalse(RepoViewModel.canPatchWorkingCopyRowInPlace(rev: "c-wc", changes: [change("p1")]))
    }

    private func change(_ id: String, parents: [String] = [], isWorkingCopy: Bool = false) -> ChangeInfo {
        ChangeInfo(
            changeId: ShortId(id: "c-\(id)", shortLen: 1),
            commitId: ShortId(id: id, shortLen: 1),
            description: "",
            author: .tester,
            parents: parents,
            bookmarks: [],
            tags: [],
            isWorkingCopy: isWorkingCopy,
            hasConflict: false,
            isEmpty: false,
            isImmutable: false,
            isDivergent: false
        )
    }
}
