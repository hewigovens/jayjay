@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoViewModelTests: RepoViewModelTestCase {
    func testApplyingSingleSelectionClearsMultiSelectionAndComparison() throws {
        let viewModel = try XCTUnwrap(viewModel)
        let detail = try viewModel.repo.showSummary(rev: "@")
        viewModel.selectedChangeIds = ["first", "second"]
        viewModel.compareFromId = "first"
        viewModel.compareToId = "second"

        viewModel.applySingleSelectedChange(detail)

        XCTAssertEqual(viewModel.selectedChangeIds, [detail.info.selectionRevision])
        XCTAssertNil(viewModel.compareFromId)
        XCTAssertNil(viewModel.compareToId)
    }

    func testDraftSurvivesMoveToEmptyWorkingCopy() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.applyWorkingCopy(changeId: "old", isDivergent: false, description: "")
        viewModel.commitSummaryDraft = "Typed summary"
        viewModel.commitDescriptionDraft = "Typed details"

        viewModel.applyWorkingCopy(changeId: "new", isDivergent: false, description: "")

        XCTAssertEqual(viewModel.commitSummaryDraft, "Typed summary")
        XCTAssertEqual(viewModel.commitDescriptionDraft, "Typed details")
    }

    func testDraftIsReplacedByDescribedWorkingCopy() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.applyWorkingCopy(changeId: "old", isDivergent: false, description: "")
        viewModel.commitSummaryDraft = "Typed summary"
        viewModel.commitDescriptionDraft = "Typed details"

        viewModel.applyWorkingCopy(
            changeId: "new",
            isDivergent: false,
            description: "Incoming summary\n\nIncoming details"
        )

        XCTAssertEqual(viewModel.commitSummaryDraft, "Incoming summary")
        XCTAssertEqual(viewModel.commitDescriptionDraft, "Incoming details")
    }

    func testNewChangeClearsCommitBox() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.commitSummaryDraft = "Previous summary"
        viewModel.commitDescriptionDraft = "Previous details"
        let previousSignal = viewModel.successActionSignal

        viewModel.newChange(parent: "@")

        for _ in 0 ..< 100 where viewModel.successActionSignal == previousSignal {
            try await Task.sleep(for: .milliseconds(20))
        }
        XCTAssertGreaterThan(viewModel.successActionSignal, previousSignal)
        XCTAssertEqual(viewModel.commitSummaryDraft, "")
        XCTAssertEqual(viewModel.commitDescriptionDraft, "")
    }

    func testKeyboardSelectionKeepsCurrentDetailUntilSettledChangeLoads() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let currentDetail = try viewModel.repo.showSummary(rev: "root()")
        viewModel.applySingleSelectedChange(currentDetail)

        viewModel.select(changeId: "root()", coalescing: true)
        viewModel.select(changeId: "@", coalescing: true)
        XCTAssertEqual(viewModel.selectedChangeId, "@")
        XCTAssertEqual(viewModel.selectedChange?.info.commitId, currentDetail.info.commitId)

        for _ in 0 ..< 200 where viewModel.selectedChange?.info.isWorkingCopy != true {
            try await Task.sleep(for: .milliseconds(20))
        }
        let detail = try XCTUnwrap(viewModel.selectedChange)
        XCTAssertTrue(detail.info.isWorkingCopy, "the earlier root() load must not win over the settled selection")
        XCTAssertEqual(viewModel.selectedChangeId, detail.info.selectionRevision)
    }

    func testNonConsecutiveSelectionClearsSingleChangePresentation() throws {
        let viewModel = try XCTUnwrap(viewModel)
        try viewModel.repo.newChange(parent: "@", message: "middle")
        try viewModel.repo.newChange(parent: "@", message: "newest")
        viewModel.graphEntries = try viewModel.repo.logGraph(revset: "all()")
        XCTAssertGreaterThanOrEqual(viewModel.changes.count, 3)
        guard viewModel.changes.count >= 3 else { return }

        let first = viewModel.changes[0].selectionRevision
        let third = viewModel.changes[2].selectionRevision
        viewModel.selectedChangeId = first
        viewModel.selectedChangeIds = [first]
        viewModel.evologRev = first
        viewModel.evologEntries = []
        viewModel.prInfo = PrInfo(
            number: 7,
            state: .open,
            title: "Previous change",
            url: "https://example.com/pr/7",
            checks: .none
        )
        let prFetchTask = Task<Void, Never> { _ = try? await Task.sleep(for: .seconds(30)) }
        viewModel.prFetchTask = prFetchTask

        viewModel.toggleSelection(changeId: third)

        XCTAssertEqual(viewModel.selectedChangeIds.count, 2)
        XCTAssertNil(viewModel.compareFromId)
        XCTAssertNil(viewModel.evologRev)
        XCTAssertNil(viewModel.evologEntries)
        XCTAssertNil(viewModel.prInfo)
        XCTAssertNil(viewModel.prFetchTask)
        XCTAssertTrue(prFetchTask.isCancelled)
    }

    func testCombinedComparisonCannotReverse() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.compareFromId = "roots"
        viewModel.compareToId = "heads"
        viewModel.compareDisplay = CompareDisplay(
            title: "2 Changes Selected",
            from: "oldest",
            to: "newest"
        )
        viewModel.selectedChangeIds = ["newest", "oldest"]

        XCTAssertFalse(viewModel.canReverseCompare)
        viewModel.reverseCompare()

        XCTAssertEqual(viewModel.compareFromId, "roots")
        XCTAssertEqual(viewModel.compareToId, "heads")
        XCTAssertEqual(viewModel.compareDisplay?.from, "oldest")
        XCTAssertEqual(viewModel.compareDisplay?.to, "newest")
    }
}
