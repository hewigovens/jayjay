@testable import JayJay
import JayJayCore
import XCTest

final class EvologViewModelTests: XCTestCase {
    func testSelectingACollapsedRunUsesItsNewestSnapshot() {
        let viewModel = makeSnapshotViewModel()
        viewModel.select(EvologRow(start: 1, count: 12), click: .replace)
        XCTAssertEqual(viewModel.selectedIndex, 1)
        XCTAssertEqual(viewModel.selectedFromCommitId, "c1")
        XCTAssertEqual(viewModel.displayedRows.count, 14)

        viewModel.setHideSnapshots(false)
        viewModel.setHideSnapshots(true)
        XCTAssertEqual(viewModel.displayedRows.count, 3, "hiding again collapses the expanded run")
    }

    func testHidingRetargetsAMiddleSnapshotSelectionToTheRunNewest() {
        let viewModel = makeSnapshotViewModel()
        viewModel.setHideSnapshots(false)
        viewModel.select(EvologRow(start: 7, count: 1), click: .replace)
        viewModel.setHideSnapshots(true)
        XCTAssertEqual(viewModel.selectedIndex, 1)
        XCTAssertEqual(viewModel.selectedFromCommitId, "c1")
    }

    func testSelectedVersionsDefineInterdiffEndpoints() {
        let entries = (0 ... 3).map { index in
            EvologEntry(
                changeId: ShortId(id: "change", shortLen: 1),
                commitId: ShortId(id: "commit-\(index)", shortLen: 1),
                timestampMillis: 0,
                operation: "rewrite",
                description: "version \(index)"
            )
        }
        let viewModel = EvologViewModel(
            entries: entries,
            changeId: "change",
            repo: nil,
            diffStore: DiffStore()
        )

        viewModel.select(EvologRow(start: 2, count: 1), click: .replace)
        XCTAssertEqual(viewModel.selectedFromCommitId, "commit-2")
        XCTAssertEqual(viewModel.selectedToCommitId, "commit-0")

        viewModel.select(EvologRow(start: 1, count: 1), click: .toggle)
        XCTAssertEqual(viewModel.selectedFromCommitId, "commit-2")
        XCTAssertEqual(viewModel.selectedToCommitId, "commit-1")

        viewModel.select(EvologRow(start: 3, count: 1), click: .extend)
        XCTAssertEqual(viewModel.selection.orderedIDs(in: Array(entries.indices)), [1, 2, 3])
        XCTAssertEqual(viewModel.selectedFromCommitId, "commit-3")
        XCTAssertEqual(viewModel.selectedToCommitId, "commit-1")
    }
}

private func makeSnapshotViewModel() -> EvologViewModel {
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
