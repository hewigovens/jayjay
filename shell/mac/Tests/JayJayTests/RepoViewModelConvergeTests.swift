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
        try jj(viewModel, "--at-op", operation, "describe", "-m", "right")

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

    func testConflictingConvergenceQueuesMergeEditorForSuccessor() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let file = URL(fileURLWithPath: viewModel.repoPath).appendingPathComponent("configuration.txt")
        try "base\n".write(to: file, atomically: true, encoding: .utf8)
        try viewModel.repo.describe(rev: "@", message: "original")
        try jj(viewModel, "new", "@-", "-m", "right source")
        try "right\n".write(to: file, atomically: true, encoding: .utf8)
        try jj(viewModel, "edit", "subject(original)")
        let operation = try XCTUnwrap(viewModel.repo.opLog().first).id.id
        try "left\n".write(to: file, atomically: true, encoding: .utf8)
        try viewModel.repo.describe(rev: "@", message: "left")
        try jj(viewModel, "--at-op", operation, "restore", "--from", "subject('right source')", "--into", "subject(original)")

        viewModel.applyFilter("divergent()", selecting: "subject(left)")
        let review = ConvergeReview()
        viewModel.previewConvergence(revision: "subject(left)", review: review)
        try await waitUntil("preview loads") { !review.isLoading }
        XCTAssertEqual(review.expectedConflicts, ["configuration.txt"])
        XCTAssertTrue(review.willConflict)
        review.customDescription = "combined"

        var applied = false
        viewModel.applyConvergence(review: review) { applied = true }
        try await waitUntil("convergence applies") { applied }
        let successor = try XCTUnwrap(viewModel.pendingConflictEditor).commitId
        try await waitUntil("successor shown outside the divergent() filter") {
            viewModel.revsetFilter.revset != "divergent()" && viewModel.selectedChange?.info.commitId.id == successor
        }
        XCTAssertNil(viewModel.takePendingConflictEditor(commitId: "other"))
        XCTAssertEqual(viewModel.takePendingConflictEditor(commitId: successor), "configuration.txt")
        XCTAssertNil(viewModel.takePendingConflictEditor(commitId: successor))
    }

    private func jj(_ viewModel: RepoViewModel, _ arguments: String...) throws {
        let process = Process()
        process.executableURL = try URL(fileURLWithPath: XCTUnwrap(findBinary(name: "jj")))
        process.arguments = ["-R", viewModel.repoPath] + arguments
        try process.run()
        process.waitUntilExit()
        XCTAssertEqual(process.terminationStatus, 0, arguments.joined(separator: " "))
    }
}
