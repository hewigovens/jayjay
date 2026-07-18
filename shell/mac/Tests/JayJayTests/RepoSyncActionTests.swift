@testable import JayJay
import XCTest

@MainActor
final class RepoSyncActionTests: RepoViewModelTestCase {
    func testRejectedPushShowsFeedback() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.isPushingInFlight = true

        XCTAssertFalse(viewModel.gitPushIfIdle(bookmark: "main"))
        XCTAssertEqual(viewModel.info, "Push already in progress")
    }

    func testRejectedPullShowsFeedback() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.isPullingInFlight = true

        viewModel.gitFetch()

        XCTAssertEqual(viewModel.info, "Pull already in progress")
    }

    func testRejectedPendingPushKeepsBookmark() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.pendingPushBookmark = "main"
        viewModel.isPushingInFlight = true

        viewModel.confirmPendingPush()

        XCTAssertEqual(viewModel.pendingPushBookmark, "main")
        XCTAssertEqual(viewModel.info, "Push already in progress")
    }
}
