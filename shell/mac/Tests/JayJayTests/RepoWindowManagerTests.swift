import AppKit
@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoWindowManagerTests: XCTestCase {
    func testOverlappingRemovalsKeepRepositoryClosedUntilLastCompletion() async throws {
        _ = NSApplication.shared
        let suiteName = "RepoWindowManagerTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suiteName))
        defer { defaults.removePersistentDomain(forName: suiteName) }
        let manager = RepoWindowManager(settings: AppSettings(defaults: defaults))
        var openedPaths: [String] = []
        manager.setWindowActions(
            openRepo: { openedPaths.append($0) },
            showRepoList: { _ in }
        )
        let path = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-overlapping-removals-\(UUID().uuidString)")
            .path

        await manager.closeRepoWindowForWorkspaceRemoval(at: path)
        await manager.closeRepoWindowForWorkspaceRemoval(at: path)
        manager.finishWorkspaceRemoval(at: path)
        manager.openRepo(path)

        XCTAssertTrue(openedPaths.isEmpty)

        manager.finishWorkspaceRemoval(at: path)
        manager.openRepo(path)

        XCTAssertEqual(openedPaths.count, 1)
    }

    func testVanishedWorkspaceCloseWaitsForRepoTasksAndReleasesOpenBarrier() async throws {
        _ = NSApplication.shared
        let suiteName = "RepoWindowManagerTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suiteName))
        defer { defaults.removePersistentDomain(forName: suiteName) }
        let manager = RepoWindowManager(settings: AppSettings(defaults: defaults))
        var openedPaths: [String] = []
        manager.setWindowActions(
            openRepo: { openedPaths.append($0) },
            showRepoList: { _ in }
        )
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-vanished-workspace-close-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        try initJjGitRepo(path: directory.path)
        let result = await manager.loadRepoViewModel(
            at: directory.path,
            includeSubmoduleStatuses: false
        )
        let viewModel = try XCTUnwrap(result.viewModel, result.error ?? "load repo view model")
        let completed = RepoWindowManagerLockedFlag()
        viewModel.runRepoTask { _ in
            Thread.sleep(forTimeInterval: 0.2)
            completed.set()
        } onSuccess: { _, _ in }

        await manager.closeRepoWindowAfterWorkspaceVanished(at: directory.path)

        XCTAssertTrue(completed.isSet, "the vanished-workspace close returned before repo work finished")
        XCTAssertTrue(viewModel.isShuttingDown)
        manager.openRepo(directory.path)
        XCTAssertEqual(openedPaths.count, 1)
    }
}

private final class RepoWindowManagerLockedFlag: @unchecked Sendable {
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
