@testable import JayJay
import JayJayCore
import XCTest

final class EvologViewModelTests: XCTestCase {
    func testSelectingACollapsedRunUsesItsNewestSnapshot() {
        let viewModel = makeViewModel()
        viewModel.select(EvologRow(start: 1, count: 12))
        XCTAssertEqual(viewModel.selectedIndex, 1)
        XCTAssertEqual(viewModel.selectedFromCommitId, "c1")
        XCTAssertEqual(viewModel.displayedRows.count, 14)

        viewModel.setHideSnapshots(false)
        viewModel.setHideSnapshots(true)
        XCTAssertEqual(viewModel.displayedRows.count, 3, "hiding again collapses the expanded run")
    }

    func testHidingRetargetsAMiddleSnapshotSelectionToTheRunNewest() {
        let viewModel = makeViewModel()
        viewModel.setHideSnapshots(false)
        viewModel.selectedIndex = 7
        viewModel.setHideSnapshots(true)
        XCTAssertEqual(viewModel.selectedIndex, 1)
        XCTAssertEqual(viewModel.selectedFromCommitId, "c1")
    }
}

private func makeViewModel() -> EvologViewModel {
    let operations = ["squash commits abc"]
        + Array(repeating: "snapshot working copy", count: 12)
        + ["describe commit def"]
    let entries = operations.enumerated().map { index, operation in
        EvologEntry(
            changeId: ShortId(id: "change", shortLen: 1),
            commitId: ShortId(id: "c\(index)", shortLen: 1),
            timestampMillis: Int64(index),
            operation: operation,
            description: ""
        )
    }
    return EvologViewModel(entries: entries, changeId: "change", repo: nil, diffStore: DiffStore())
}
