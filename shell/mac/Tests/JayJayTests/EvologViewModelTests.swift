@testable import JayJay
import JayJayCore
import XCTest

final class EvologViewModelTests: XCTestCase {
    func testCollapsesConsecutiveSnapshotRuns() {
        let viewModel = makeViewModel(entries: [
            entry(commitId: "a", operation: "snapshot working copy"),
            entry(commitId: "b", operation: "snapshot working copy"),
            entry(commitId: "c", operation: "describe commit c"),
            entry(commitId: "d", operation: "snapshot working copy")
        ])

        XCTAssertEqual(viewModel.visibleRows.count, 3)
        XCTAssertEqual(viewModel.visibleRows[0].entries.count, 2)
        XCTAssertTrue(viewModel.visibleRows[0].isSnapshotRun)
        XCTAssertEqual(viewModel.visibleRows[1].primary.commitId, "c")
        XCTAssertFalse(viewModel.visibleRows[2].isSnapshotRun)
    }

    func testHideSnapshotsRemovesSnapshotRows() {
        let viewModel = makeViewModel(entries: [
            entry(commitId: "a", operation: "snapshot working copy"),
            entry(commitId: "b", operation: "describe commit b"),
            entry(commitId: "c", operation: "snapshot working copy")
        ])

        viewModel.setHideSnapshots(true)

        XCTAssertEqual(viewModel.visibleRows.map(\.primary.commitId), ["b"])
        XCTAssertEqual(viewModel.hiddenSnapshotCount, 2)
    }

    func testNormalizeSelectionClearsHiddenSnapshotSelection() {
        let viewModel = makeViewModel(entries: [
            entry(commitId: "a", operation: "snapshot working copy"),
            entry(commitId: "b", operation: "describe commit b")
        ])
        viewModel.selectedIndex = 0

        viewModel.setHideSnapshots(true)

        XCTAssertNil(viewModel.selectedIndex)
        XCTAssertNil(viewModel.interdiffDetail)
    }

    func testNormalizeSelectionClearsNegativeSelection() {
        let viewModel = makeViewModel(entries: [
            entry(commitId: "a", operation: "describe commit a")
        ])
        viewModel.selectedIndex = -1

        viewModel.normalizeSelection()

        XCTAssertNil(viewModel.selectedIndex)
    }

    func testNormalizeSelectionMovesCollapsedSnapshotSelectionToVisiblePrimary() {
        let viewModel = makeViewModel(entries: [
            entry(commitId: "a", operation: "snapshot working copy"),
            entry(commitId: "b", operation: "snapshot working copy"),
            entry(commitId: "c", operation: "describe commit c")
        ])
        viewModel.collapseSnapshotRuns = false
        viewModel.selectedIndex = 1

        viewModel.setCollapseSnapshotRuns(true)

        XCTAssertEqual(viewModel.selectedIndex, 0)
    }

    private func makeViewModel(entries: [EvologEntry]) -> EvologViewModel {
        EvologViewModel(entries: entries, changeId: "change", repo: nil, diffStore: DiffStore())
    }

    private func entry(commitId: String, operation: String) -> EvologEntry {
        EvologEntry(
            changeId: "change",
            commitId: commitId,
            timestampMillis: 0,
            operation: operation,
            description: ""
        )
    }
}
