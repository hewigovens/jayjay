@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoFSWatcherTests: XCTestCase {
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
}
