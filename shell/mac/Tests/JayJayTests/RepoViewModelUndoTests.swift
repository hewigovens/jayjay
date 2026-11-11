@testable import JayJay
import JayJayCore
import XCTest

final class RepoViewModelUndoTests: RepoViewModelTestCase {
    func testNoticeUndoRevertsTheOperationTheActionProduced() async throws {
        let (viewModel, doomed) = try abandonFixture()
        let notice = try await abandon(doomed, in: viewModel)
        XCTAssertEqual(notice.undoOperationId, viewModel.repo.undoState().operationId)
        let successSignal = viewModel.successActionSignal

        try viewModel.undo(operationId: XCTUnwrap(notice.undoOperationId))
        try await waitUntil("the undo finishes") { viewModel.info != nil }
        XCTAssertEqual(viewModel.successActionSignal, successSignal, "undo must not count toward the rating prompt")

        XCTAssertEqual(try viewModel.repo.log(revset: "subject(exact:doomed)").count, 1)
        XCTAssertNil(viewModel.operationLogRequest)
    }

    func testNoticeUndoAfterLaterOperationsOpensTheOperationLogInstead() async throws {
        let (viewModel, doomed) = try abandonFixture()
        let notice = try await abandon(doomed, in: viewModel)
        try viewModel.repo.describe(rev: "@", message: "later")
        let operationId = try XCTUnwrap(notice.undoOperationId)

        viewModel.undo(operationId: operationId)
        try await waitUntil("the undo declines") { viewModel.operationLogRequest != nil }

        XCTAssertEqual(viewModel.operationLogRequest, operationId)
        XCTAssertTrue(try viewModel.repo.log(revset: "subject(exact:doomed)").isEmpty)
    }

    private func abandonFixture() throws -> (RepoViewModel, String) {
        let viewModel = try XCTUnwrap(viewModel)
        try viewModel.repo.newChange(parent: "@", message: "doomed")
        try viewModel.repo.newChange(parent: "@", message: "")
        let doomed = try viewModel.repo.showSummary(rev: "subject(exact:doomed)").info.changeId.id
        return (viewModel, doomed)
    }

    private func abandon(_ changeId: String, in viewModel: RepoViewModel) async throws -> RepoActionNotice {
        viewModel.abandon(rev: changeId)
        try await waitUntil("the abandon finishes") { viewModel.actionNotice != nil }
        return try XCTUnwrap(viewModel.actionNotice)
    }
}
