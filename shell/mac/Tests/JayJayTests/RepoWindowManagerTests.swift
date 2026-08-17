import AppKit
@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoWindowManagerTests: XCTestCase {
    private func makeManager(openedPaths: @escaping (String) -> Void) throws -> RepoWindowManager {
        _ = NSApplication.shared
        let suiteName = "RepoWindowManagerTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suiteName))
        addTeardownBlock { defaults.removePersistentDomain(forName: suiteName) }
        let manager = RepoWindowManager(settings: AppSettings(defaults: defaults))
        manager.setWindowActions(openRepo: openedPaths, showRepoList: { _ in })
        return manager
    }

    func testOverlappingRemovalsKeepRepositoryClosedUntilLastCompletion() async throws {
        var openedPaths: [String] = []
        let manager = try makeManager { openedPaths.append($0) }
        let path = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-overlapping-removals-\(UUID().uuidString)")
            .path

        await manager.withWorkspaceRemoval(at: path) {
            await manager.withWorkspaceRemoval(at: path) {}
            manager.openRepo(path)
            XCTAssertTrue(openedPaths.isEmpty, "an inner removal's completion released the outer one's barrier")
        }
        manager.openRepo(path)

        XCTAssertEqual(openedPaths.count, 1)
    }

    func testRemovalWaitsForRepoTasksAndReleasesOpenBarrier() async throws {
        var openedPaths: [String] = []
        let manager = try makeManager { openedPaths.append($0) }
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-vanished-workspace-close-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        try initJjGitRepo(path: directory.path)
        let repo = try JayJayRepo.open(path: directory.path)
        let viewModel = RepoViewModel(
            path: directory.path,
            repo: repo,
            workingCopyIsLarge: false,
            configWarning: nil,
            includeSubmoduleStatuses: false
        )
        XCTAssertTrue(manager.register(viewModel))
        let completed = LockedFlag()
        viewModel.runRepoTask { _ in
            completed.setAfterBlocking(seconds: 0.2)
        } onSuccess: { _, _ in }

        await manager.withWorkspaceRemoval(at: directory.path) {
            XCTAssertTrue(completed.isSet, "the removal body ran before repo work finished")
            XCTAssertTrue(viewModel.isShuttingDown)
            XCTAssertFalse(manager.register(viewModel), "a window opening mid-removal must not register")
        }
        manager.openRepo(directory.path)

        XCTAssertEqual(openedPaths.count, 1)
    }
}
