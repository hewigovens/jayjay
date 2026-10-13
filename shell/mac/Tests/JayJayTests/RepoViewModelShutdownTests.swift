import Foundation
@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoViewModelShutdownTests: RepoViewModelTestCase {
    func testPrepareForRemovalOutwaitsEveryKindOfRepoTask() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let callback = LockedFlag()
        let awaited = LockedFlag()
        let superseded = LockedFlag()
        viewModel.runRepoTask { _ in callback.setAfterBlocking(seconds: 0.2) } onSuccess: { _, _ in }
        let awaitedStarted = LockedFlag()
        let operation = Task { @MainActor in
            try await viewModel.awaitRepoTask { _ in
                awaitedStarted.set()
                awaited.setAfterBlocking(seconds: 0.2)
            }
        }
        while !awaitedStarted.isSet {
            await Task.yield()
        }
        let stale = viewModel.startRepoTask { superseded.setAfterBlocking(seconds: 0.2) }
        viewModel.refreshTask = stale
        stale.cancel()
        viewModel.refreshTask = viewModel.startRepoTask {}

        await viewModel.prepareForRemoval()

        try await operation.value
        XCTAssertTrue(callback.isSet && awaited.isSet && superseded.isSet, "prepareForRemoval returned before repo work finished")
        XCTAssertTrue(viewModel.repoTasks.isEmpty)
    }

    func testShutdownRefusesNewRepoWork() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        await viewModel.prepareForRemoval()

        let ran = LockedFlag()
        viewModel.runRepoTask { _ in ran.set() } onSuccess: { _, _ in }
        viewModel.refresh()
        viewModel.fetchPrInfo(bookmarks: ["feature"])
        do {
            _ = try await viewModel.awaitRepoTask { _ in () }
            XCTFail("an awaited repo task started after the shutdown barrier")
        } catch is CancellationError {}

        try await Task.sleep(for: .milliseconds(100))
        XCTAssertFalse(ran.isSet)
        XCTAssertNil(viewModel.refreshTask)
        XCTAssertNil(viewModel.prFetchTask)
        XCTAssertTrue(viewModel.repoTasks.isEmpty)
    }

    func testClosingTheWindowCancelsADispatchedBisectStart() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let repo = viewModel.repo
        try repo.describe(rev: "@", message: "good")
        let good = try XCTUnwrap(repo.log(revset: "@").first).changeId.id
        try repo.newChange(parent: "@", message: "candidate")
        try repo.newChange(parent: "@", message: "bad")
        let original = try XCTUnwrap(repo.log(revset: "@").first).changeId.id
        let dispatched = LockedFlag()
        let resume = DispatchSemaphore(value: 0)
        defer { resume.signal() }
        viewModel.runRepoTask { repo in
            dispatched.set()
            resume.wait()
            return try repo.startBisectBetween(first: good, second: original)
        } onSuccess: { _, _ in }
        try await waitUntil("bisect start is dispatched") { dispatched.isSet }
        XCTAssertNil(repo.bisectState())

        viewModel.windowWillClose()
        resume.signal()
        try await waitUntil("pending start finishes") { viewModel.repoTasks.isEmpty }

        XCTAssertNil(repo.bisectState())
        XCTAssertEqual(try XCTUnwrap(repo.log(revset: "@").first).changeId.id, original)
    }
}
