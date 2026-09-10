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
        XCTAssertEqual(viewModel.pendingBackgroundRefresh, .reload)

        viewModel.setBackgroundRefreshSuspended(false)
        XCTAssertTrue(viewModel.isRefreshingInFlight)
        viewModel.setBackgroundRefreshSuspended(true)

        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }
        XCTAssertEqual(viewModel.pendingBackgroundRefresh, .reload)
        XCTAssertFalse(viewModel.selectedChange?.diff.contains { $0.path == "late-edit.txt" } == true)

        viewModel.setBackgroundRefreshSuspended(false)
        XCTAssertTrue(viewModel.isRefreshingInFlight)
        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }
        XCTAssertNil(viewModel.error)
        XCTAssertTrue(viewModel.selectedChange?.diff.contains { $0.path == "late-edit.txt" } == true)
    }

    func testOperationEventIsDroppedOnlyWhileTheRepoIsAtHead() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        try await snapshotAnEdit(viewModel, file: "echo.txt")

        viewModel.handleOperationChange()
        XCTAssertFalse(viewModel.isRefreshingInFlight)
        XCTAssertNil(viewModel.pendingBackgroundRefresh)

        viewModel.isRefreshingInFlight = true
        viewModel.handleOperationChange()
        viewModel.isRefreshingInFlight = false
        viewModel.resumePendingBackgroundRefresh()
        XCTAssertFalse(viewModel.isRefreshingInFlight)
        XCTAssertNil(viewModel.pendingBackgroundRefresh)

        try "from elsewhere\n".write(
            to: URL(fileURLWithPath: viewModel.repoPath).appending(path: "external.txt"),
            atomically: true,
            encoding: .utf8
        )
        try JayJayRepo.open(path: viewModel.repoPath).refreshWorkingCopy()
        viewModel.lastInternalMutationAt = Date()
        viewModel.handleOperationChange()
        XCTAssertTrue(viewModel.isRefreshingInFlight)
        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }
        XCTAssertTrue(viewModel.selectedChange?.diff.contains { $0.path == "external.txt" } == true)
    }

    func testFailedReloadRetainsPendingOperationEvenAtHead() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        try await snapshotAnEdit(viewModel, file: "before.txt")

        for suspended in [false, true] {
            let file = "after-\(suspended).txt"
            try "new snapshot\n".write(
                to: URL(fileURLWithPath: viewModel.repoPath).appending(path: file),
                atomically: true,
                encoding: .utf8
            )
            viewModel.isRefreshingInFlight = true
            try viewModel.repo.refreshWorkingCopy()
            XCTAssertTrue(try viewModel.repo.isAtOperationHead())
            XCTAssertFalse(viewModel.selectedChange?.diff.contains { $0.path == file } == true)
            viewModel.handleOperationChange()
            viewModel.lastInternalMutationAt = Date()
            viewModel.setBackgroundRefreshSuspended(suspended)

            viewModel.applyRefreshFailure(TestRefreshError.failed, presence: viewModel.repo.workspacePresence())

            if suspended {
                XCTAssertFalse(viewModel.isRefreshingInFlight)
                XCTAssertEqual(viewModel.pendingBackgroundRefresh, .reload)
                viewModel.setBackgroundRefreshSuspended(false)
            }
            XCTAssertTrue(viewModel.isRefreshingInFlight)
            try await waitUntil("the retry finishes") { !viewModel.isRefreshingInFlight }
            XCTAssertTrue(viewModel.selectedChange?.diff.contains { $0.path == file } == true)
            XCTAssertNotNil(viewModel.error)
            XCTAssertNil(viewModel.pendingBackgroundRefresh)
        }

        viewModel.applyRefreshFailure(TestRefreshError.failed, presence: .exists)
        XCTAssertFalse(viewModel.isRefreshingInFlight)
    }

    func testQueuedWorkingCopyAndOperationEventsShareOneRefresh() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        try await snapshotAnEdit(viewModel, file: "before.txt")

        for operationFirst in [false, true] {
            let file = "queued-\(operationFirst).txt"
            try "queued edit\n".write(
                to: URL(fileURLWithPath: viewModel.repoPath).appending(path: file),
                atomically: true,
                encoding: .utf8
            )
            try JayJayRepo.open(path: viewModel.repoPath).refreshWorkingCopy()
            XCTAssertFalse(try viewModel.repo.isAtOperationHead())
            viewModel.isRefreshingInFlight = true
            if operationFirst {
                viewModel.handleOperationChange()
                viewModel.handleWorkingCopyChange()
            } else {
                viewModel.handleWorkingCopyChange()
                viewModel.handleOperationChange()
            }
            viewModel.isRefreshingInFlight = false
            viewModel.resumePendingBackgroundRefresh()

            XCTAssertTrue(viewModel.isRefreshingInFlight)
            XCTAssertNil(viewModel.pendingBackgroundRefresh)
            try await waitUntil("the queued refresh finishes") { !viewModel.isRefreshingInFlight }
            XCTAssertTrue(viewModel.selectedChange?.diff.contains { $0.path == file } == true)
        }
    }

    func testSelectionChangedDuringARefreshSurvivesIt() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.refresh()
        viewModel.selectedChangeIds = ["first", "second"]
        viewModel.selectedChangeId = "second"

        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }

        XCTAssertFalse(viewModel.graphEntries.isEmpty)
        XCTAssertEqual(viewModel.selectedChangeIds, ["first", "second"])
    }

    func testSelectionMadeAfterTheFirstPaintSurvivesTheSnapshotPass() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.refresh(selecting: "@")
        try await waitUntil("the first paint lands") { !viewModel.graphEntries.isEmpty }
        viewModel.selectedChangeIds = ["first", "second"]
        viewModel.selectedChangeId = "second"

        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }

        XCTAssertEqual(viewModel.selectedChangeIds, ["first", "second"])
    }

    func testWorkingCopyEventIsNeverTakenForOurSnapshotEcho() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        try await snapshotAnEdit(viewModel, file: "echo.txt")

        viewModel.handleWorkingCopyChange()
        XCTAssertTrue(viewModel.isRefreshingInFlight)
        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }
    }

    private func snapshotAnEdit(_ viewModel: RepoViewModel, file: String) async throws {
        try "snapshot me\n".write(
            to: URL(fileURLWithPath: viewModel.repoPath).appending(path: file),
            atomically: true,
            encoding: .utf8
        )
        viewModel.refresh()
        try await waitUntil("the refresh finishes") { !viewModel.isRefreshingInFlight }
        XCTAssertTrue(viewModel.selectedChange?.diff.contains { $0.path == file } == true)
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
