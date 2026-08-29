@testable import JayJay
import JayJayCore
import XCTest

final class EvologViewModelTests: XCTestCase {
    func testHidingGroupsConsecutiveSnapshots() {
        let entries = describeThenSnapshotsThenSquash()
        XCTAssertEqual(
            EvologViewModel.displayedRows(entries: entries, hideSnapshots: true),
            [.entry(0), .collapsedRun(1 ..< 13), .entry(13)]
        )
        XCTAssertEqual(
            EvologViewModel.displayedRows(entries: entries, hideSnapshots: false).count,
            14
        )
        XCTAssertEqual(
            EvologViewModel.displayedRows(entries: entries, hideSnapshots: true, expandedRuns: [1]).count,
            14
        )
        XCTAssertEqual(
            EvologViewModel.displayedRows(
                entries: evologEntries(Array(repeating: "snapshot working copy", count: 3)),
                hideSnapshots: true
            ),
            [.entry(0), .collapsedRun(1 ..< 3)]
        )
        XCTAssertEqual(
            EvologViewModel.displayedRows(
                entries: evologEntries(["squash", "snapshot working copy", "describe"]),
                hideSnapshots: true
            ),
            [.entry(0), .entry(1), .entry(2)]
        )
        XCTAssertEqual(
            EvologViewModel.displayedRows(
                entries: evologEntries([
                    "squash",
                    "snapshot working copy",
                    "snapshot working copy",
                    "describe",
                    "snapshot working copy",
                    "snapshot working copy",
                    "snapshot working copy"
                ]),
                hideSnapshots: true
            ),
            [.entry(0), .collapsedRun(1 ..< 3), .entry(3), .collapsedRun(4 ..< 7)]
        )
    }

    func testSelectingACollapsedRunUsesItsNewestSnapshot() {
        let viewModel = EvologViewModel(
            entries: describeThenSnapshotsThenSquash(),
            changeId: "change",
            repo: nil,
            diffStore: DiffStore()
        )
        viewModel.select(.collapsedRun(1 ..< 13))
        XCTAssertEqual(viewModel.selectedIndex, 1)
        XCTAssertEqual(viewModel.selectedFromCommitId, "c1")
        XCTAssertEqual(viewModel.displayedRows.count, 14)
    }

    func testHidingRetargetsAMiddleSnapshotSelectionToTheRunNewest() {
        let viewModel = EvologViewModel(
            entries: describeThenSnapshotsThenSquash(),
            changeId: "change",
            repo: nil,
            diffStore: DiffStore()
        )
        viewModel.hideSnapshots = false
        viewModel.selectedIndex = 7
        viewModel.setHideSnapshots(true)
        XCTAssertEqual(viewModel.selectedIndex, 1)
        XCTAssertEqual(viewModel.selectedFromCommitId, "c1")
    }
}

private func describeThenSnapshotsThenSquash() -> [EvologEntry] {
    evologEntries(
        ["squash commits abc"]
            + Array(repeating: "snapshot working copy", count: 12)
            + ["describe commit def"]
    )
}

private func evologEntries(_ operations: [String]) -> [EvologEntry] {
    operations.enumerated().map { index, operation in
        EvologEntry(
            changeId: ShortId(id: "change", shortLen: 1),
            commitId: ShortId(id: "c\(index)", shortLen: 1),
            timestampMillis: Int64(index),
            operation: operation,
            description: ""
        )
    }
}
