@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoViewModelConvergeTests: RepoViewModelTestCase {
    func testStaleConvergenceRefreshesPreviewAndClearsChoices() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let file = URL(fileURLWithPath: viewModel.repoPath).appendingPathComponent("configuration.txt")
        try "base\n".write(to: file, atomically: true, encoding: .utf8)
        try viewModel.repo.describe(rev: "@", message: "original")
        let operation = try XCTUnwrap(viewModel.repo.opLog().first).id.id
        try viewModel.repo.describe(rev: "@", message: "left")
        let process = Process()
        process.executableURL = try URL(fileURLWithPath: XCTUnwrap(findBinary(name: "jj")))
        process.arguments = ["-R", viewModel.repoPath, "--at-op", operation, "describe", "-m", "right"]
        try process.run()
        process.waitUntilExit()
        XCTAssertEqual(process.terminationStatus, 0)

        let review = ConvergeReview()
        viewModel.previewConvergence(revision: "@", review: review)
        try await waitUntil("preview loads") { !review.isLoading }
        let basis = try XCTUnwrap(review.plan).basis
        review.customDescription = "combined"
        XCTAssertTrue(review.canApply)
        try "changed after preview\n".write(to: file, atomically: true, encoding: .utf8)

        var applied = false
        viewModel.applyConvergence(review: review) { applied = true }
        try await waitUntil("stale preview refreshes") { !review.isApplying && !review.isLoading }
        XCTAssertFalse(applied)
        XCTAssertTrue(review.wasStale)
        XCTAssertNil(review.error)
        XCTAssertTrue(review.isCurrent)
        XCTAssertNotEqual(review.plan?.basis, basis)
        XCTAssertNil(review.customDescription)
        XCTAssertTrue(review.selections.isEmpty)
        XCTAssertFalse(review.canApply)
    }
}
