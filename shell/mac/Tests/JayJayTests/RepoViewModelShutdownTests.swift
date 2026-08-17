@testable import JayJay
import XCTest

@MainActor
final class RepoViewModelShutdownTests: RepoViewModelTestCase {
    /// Deleting a workspace moves its directory; the barrier must outwait any repo task that could still be reading or mutating the checkout.
    func testPrepareForRemovalWaitsForInFlightRepoTasks() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let completed = LockedFlag()
        viewModel.runRepoTask { _ in
            Thread.sleep(forTimeInterval: 0.2)
            completed.set()
        } onSuccess: {
            _, _ in
        }

        await viewModel.prepareForRemoval()

        XCTAssertTrue(completed.isSet, "prepareForRemoval returned before an in-flight repo task finished")
        XCTAssertTrue(viewModel.inFlightRepoTasks.isEmpty)
    }

    func testPrepareForRemovalWaitsForAwaitedRepoTasks() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let started = LockedFlag()
        let completed = LockedFlag()
        let operation = Task { @MainActor in
            try await viewModel.awaitRepoTask { _ in
                started.set()
                Thread.sleep(forTimeInterval: 0.2)
                completed.set()
            }
        }
        while !started.isSet {
            await Task.yield()
        }

        await viewModel.prepareForRemoval()

        try await operation.value
        XCTAssertTrue(completed.isSet, "prepareForRemoval returned before an awaited repo task finished")
        XCTAssertTrue(viewModel.inFlightRepoTasks.isEmpty)
    }

    func testPrepareForRemovalWaitsForSupersededLifecycleTasks() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let firstCompleted = LockedFlag()
        let secondCompleted = LockedFlag()
        let first = viewModel.startLifecycleRepoTask {
            Thread.sleep(forTimeInterval: 0.2)
            firstCompleted.set()
        }
        viewModel.refreshTask = first
        first.cancel()
        viewModel.refreshTask = viewModel.startLifecycleRepoTask {
            Thread.sleep(forTimeInterval: 0.1)
            secondCompleted.set()
        }

        await viewModel.prepareForRemoval()

        XCTAssertTrue(firstCompleted.isSet, "prepareForRemoval returned before superseded lifecycle work finished")
        XCTAssertTrue(secondCompleted.isSet, "prepareForRemoval returned before the latest lifecycle work finished")
        XCTAssertTrue(viewModel.lifecycleRepoTasks.isEmpty)
    }

    func testShutdownRefusesNewRepoWork() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        await viewModel.prepareForRemoval()

        let ran = LockedFlag()
        viewModel.runRepoTask {
            _ in ran.set()
        } onSuccess: {
            _, _ in
        }
        viewModel.refresh()

        try await Task.sleep(for: .milliseconds(100))
        XCTAssertFalse(ran.isSet, "a repo task started after the shutdown barrier")
        XCTAssertTrue(viewModel.inFlightRepoTasks.isEmpty)
        XCTAssertNil(viewModel.refreshTask)
    }

    func testShutdownRefusesNewPullRequestFetches() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        await viewModel.prepareForRemoval()

        viewModel.fetchPrInfo(bookmarks: ["feature"])

        XCTAssertNil(viewModel.prFetchTask)
    }

    func testShutdownRefusesNewAwaitedRepoWork() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        await viewModel.prepareForRemoval()

        do {
            _ = try await viewModel.awaitRepoTask { _ in () }
            XCTFail("an awaited repo task started after the shutdown barrier")
        } catch is CancellationError {
            XCTAssertTrue(viewModel.inFlightRepoTasks.isEmpty)
        }
    }
}

/// The operation runs off the main actor, so the completion signal needs its own lock.
private final class LockedFlag: @unchecked Sendable {
    private let lock = NSLock()
    private var value = false

    func set() {
        lock.lock()
        value = true
        lock.unlock()
    }

    var isSet: Bool {
        lock.lock()
        defer { lock.unlock() }
        return value
    }
}
