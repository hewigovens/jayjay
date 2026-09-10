@testable import JayJay
import JayJayCore
import XCTest

private final class Counter: @unchecked Sendable {
    private var value = 0
    private let lock = NSLock()

    func increment() -> Int {
        lock.withLock {
            value += 1
            return value
        }
    }
}

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

        let first = expectation(description: "first operation observed")
        let second = expectation(description: "second operation observed after the debounce window")
        let count = Counter()
        let watcher = RepoFSWatcher(repoPath: directory.path, onChange: {
            (count.increment() == 1 ? first : second).fulfill()
        })
        try repo.describe(rev: "@", message: "first")
        wait(for: [first], timeout: 5)
        try repo.describe(rev: "@", message: "second, inside the debounce window")

        wait(for: [second], timeout: 5)
        withExtendedLifetime(watcher) {}
    }
}
