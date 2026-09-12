@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoFSWatcherTests: XCTestCase {
    func testWorkingCopyBatchesInsideTheLatencyWindowAreBothDelivered() {
        let observed = expectation(description: "both relevant working-copy batches delivered")
        observed.expectedFulfillmentCount = 2
        let watcher = RepoFSWatcher(
            repoPath: "/unavailable-jayjay-watcher-\(UUID().uuidString)",
            onChange: { XCTFail("working-copy events must not become operation events") },
            onWorkingCopyChange: { observed.fulfill() },
            isRelevantWorkingCopyChange: { $0.contains("tracked.txt") }
        )

        watcher.handleWorkingCopyEvents(["tracked.txt"])
        watcher.handleWorkingCopyEvents(["ignored.txt"])
        watcher.handleWorkingCopyEvents(["tracked.txt"])

        wait(for: [observed], timeout: 3)
    }

    func testSecondaryWorkspaceSeesOperationsInThePrimary() throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-watcher-\(UUID().uuidString)")
        let primary = directory.appending(path: "primary")
        try FileManager.default.createDirectory(at: primary, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        try initJjGitRepo(path: primary.path)
        let repo = try JayJayRepo.open(path: primary.path)
        let secondary = directory.appending(path: "secondary").path
        _ = try repo.workspaceAdd(dest: secondary, name: "secondary", rev: "")

        let observed = expectation(description: "operation observed from the secondary workspace")
        observed.assertForOverFulfill = false
        let watcher = RepoFSWatcher(repoPath: secondary, onChange: { observed.fulfill() })
        try repo.describe(rev: "@", message: "an operation in the primary")

        wait(for: [observed], timeout: 5)
        withExtendedLifetime(watcher) {}
    }

    func testOperationInsideTheDebounceWindowStillFires() throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-watcher-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        try initJjGitRepo(path: directory.path)
        let repo = try JayJayRepo.open(path: directory.path)

        // Operation callbacks arrive on the main queue, where this test also runs. One operation can touch the heads directory more than once, so counts are lower bounds and the watcher is left to go quiet before the operations under test.
        nonisolated(unsafe) var fired = 0
        let watcher = RepoFSWatcher(repoPath: directory.path, onChange: { fired += 1 })
        try repo.describe(rev: "@", message: "warm up the watcher")
        wait(for: [expectation(for: NSPredicate { _, _ in fired >= 1 }, evaluatedWith: nil)], timeout: 5)
        RunLoop.main.run(until: Date().addingTimeInterval(1.2))
        let quiet = fired

        try repo.describe(rev: "@", message: "first")
        wait(for: [expectation(for: NSPredicate { _, _ in fired > quiet }, evaluatedWith: nil)], timeout: 5)
        let afterFirst = fired
        try repo.describe(rev: "@", message: "second, inside the debounce window")

        wait(for: [expectation(for: NSPredicate { _, _ in fired > afterFirst }, evaluatedWith: nil)], timeout: 5)
        withExtendedLifetime(watcher) {}
    }
}
