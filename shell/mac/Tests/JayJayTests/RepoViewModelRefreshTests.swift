import Foundation
@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoViewModelRefreshTests: RepoViewModelTestCase {
    func testWorkingCopyChangeWaitsForEditingAndDefersAnInFlightResult() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.refresh()
        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }
        XCTAssertTrue(viewModel.selectedChange?.info.isWorkingCopy == true)

        try "refresh me\n".write(
            to: URL(fileURLWithPath: viewModel.repoPath).appending(path: "late-edit.txt"),
            atomically: true,
            encoding: .utf8
        )
        viewModel.setBackgroundRefreshSuspended(true)
        viewModel.lastInternalMutationAt = Date()
        viewModel.handleWorkingCopyChange()
        XCTAssertFalse(viewModel.isRefreshingInFlight)
        XCTAssertTrue(viewModel.hasPendingBackgroundRefresh)

        viewModel.setBackgroundRefreshSuspended(false)
        XCTAssertTrue(viewModel.isRefreshingInFlight)
        viewModel.setBackgroundRefreshSuspended(true)

        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }
        XCTAssertTrue(viewModel.hasPendingBackgroundRefresh)
        XCTAssertFalse(viewModel.selectedChange?.diff.contains { $0.path == "late-edit.txt" } == true)

        viewModel.setBackgroundRefreshSuspended(false)
        XCTAssertTrue(viewModel.isRefreshingInFlight)
        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }
        XCTAssertNil(viewModel.error)
        XCTAssertTrue(viewModel.selectedChange?.diff.contains { $0.path == "late-edit.txt" } == true)
    }

    func testCancelledFailureProbeCannotOverwriteNewerRefreshState() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let probe = BlockingWorkspacePresenceProbe()
        viewModel.isLoading = true
        viewModel.isRefreshingInFlight = true

        let staleRefresh = viewModel.startRepoTask { [viewModel] in
            await viewModel.handleRefreshFailure(TestRefreshError.failed) {
                probe.run()
            }
        }
        while !probe.hasStarted {
            await Task.yield()
        }

        staleRefresh.cancel()
        viewModel.isLoading = false
        viewModel.isRefreshingInFlight = false
        viewModel.error = "newer refresh"
        probe.finish()
        await staleRefresh.value

        XCTAssertFalse(viewModel.workspaceVanished)
        XCTAssertEqual(viewModel.error, "newer refresh")
    }
}

private enum TestRefreshError: Error {
    case failed
}

private final class BlockingWorkspacePresenceProbe: @unchecked Sendable {
    private let lock = NSLock()
    private let release = DispatchSemaphore(value: 0)
    private var started = false

    var hasStarted: Bool {
        lock.lock()
        defer { lock.unlock() }
        return started
    }

    func run() -> WorkspacePresence {
        lock.lock()
        started = true
        lock.unlock()
        release.wait()
        return .gone
    }

    func finish() {
        release.signal()
    }
}
