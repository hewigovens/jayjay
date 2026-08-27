@testable import JayJay
import XCTest

@MainActor
final class RepoViewModelTests: RepoViewModelTestCase {
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

    func testKeyboardSelectionLoadsTheChangeTheKeySettlesOn() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.select(changeId: "root()", coalescing: true)
        viewModel.select(changeId: "@", coalescing: true)
        XCTAssertEqual(viewModel.selectedChangeId, "@")
        XCTAssertNil(viewModel.selectedChange)

        for _ in 0 ..< 200 where viewModel.selectedChange == nil {
            try await Task.sleep(for: .milliseconds(20))
        }
        let detail = try XCTUnwrap(viewModel.selectedChange)
        XCTAssertTrue(detail.info.isWorkingCopy, "the earlier root() load must not win over the settled selection")
        XCTAssertEqual(viewModel.selectedChangeId, detail.info.selectionRevision)
    }
}
