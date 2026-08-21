import AppKit
@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoWindowManagerTests: XCTestCase {
    private struct WorkspaceRemovalFixture {
        let manager: RepoWindowManager
        let workspace: WorkspaceInfo
        let source: RepoViewModel
        let target: RepoViewModel
        let window: NSWindow
        let checkout: URL
        let movedCheckout: URL
    }

    private func makeManager(openedPaths: @escaping (String) -> Void = { _ in }) throws -> RepoWindowManager {
        _ = NSApplication.shared
        let suiteName = "RepoWindowManagerTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suiteName))
        addTeardownBlock { defaults.removePersistentDomain(forName: suiteName) }
        let manager = RepoWindowManager(settings: AppSettings(defaults: defaults))
        manager.setWindowActions(openRepo: openedPaths, showRepoList: { _ in })
        return manager
    }

    private func makeTemporaryDirectory(named name: String) throws -> URL {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-\(name)-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        addTeardownBlock { try? FileManager.default.removeItem(at: directory) }
        return directory
    }

    private func makeRepository(named name: String) throws -> (URL, JayJayRepo) {
        let directory = try makeTemporaryDirectory(named: name)
        try initJjGitRepo(path: directory.path)
        return try (directory, JayJayRepo.open(path: directory.path))
    }

    private func makeViewModel(at directory: URL, repo: JayJayRepo? = nil) throws -> RepoViewModel {
        let resolvedRepo: JayJayRepo = if let repo {
            repo
        } else {
            try JayJayRepo.open(path: directory.path)
        }
        return RepoViewModel(
            path: directory.path,
            repo: resolvedRepo,
            workingCopyIsLarge: false,
            configWarning: nil,
            includeSubmoduleStatuses: false
        )
    }

    private func makeWindow(representing url: URL) -> NSWindow {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 200, height: 100),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        window.isReleasedWhenClosed = false
        window.representedURL = url
        window.orderFront(nil)
        return window
    }

    private func makeWorkspaceRemovalFixture() throws -> WorkspaceRemovalFixture {
        let manager = try makeManager()
        let root = try makeTemporaryDirectory(named: "workspace-removal")
        let sourceDirectory = root.appending(path: "source")
        let checkout = root.appending(path: "feature")
        let alias = root.appending(path: "feature-alias")
        let movedCheckout = root.appending(path: "feature-moved")
        try FileManager.default.createDirectory(at: sourceDirectory, withIntermediateDirectories: true)
        try initJjGitRepo(path: sourceDirectory.path)

        let sourceRepo = try JayJayRepo.open(path: sourceDirectory.path)
        _ = try sourceRepo.workspaceAdd(dest: checkout.path, name: "feature", rev: "")
        try FileManager.default.createSymbolicLink(at: alias, withDestinationURL: checkout)
        let listed = try XCTUnwrap(sourceRepo.workspaceList().first { $0.name == "feature" })
        let workspace = WorkspaceInfo(
            name: listed.name,
            path: "",
            isPathResolved: false,
            isCurrent: listed.isCurrent,
            changeId: listed.changeId,
            description: listed.description,
            timestamp: listed.timestamp,
            hasConflict: listed.hasConflict,
            filesChanged: listed.filesChanged
        )
        let source = try makeViewModel(at: sourceDirectory, repo: sourceRepo)
        let target = try makeViewModel(at: alias)
        XCTAssertTrue(manager.register(target))
        return WorkspaceRemovalFixture(
            manager: manager,
            workspace: workspace,
            source: source,
            target: target,
            window: makeWindow(representing: alias),
            checkout: checkout,
            movedCheckout: movedCheckout
        )
    }

    func testOverlappingRemovalsKeepRepositoryClosedUntilLastCompletion() async throws {
        var openedPaths: [String] = []
        let manager = try makeManager { openedPaths.append($0) }
        let path = try makeTemporaryDirectory(named: "overlapping-removals").path

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
        let (directory, repo) = try makeRepository(named: "vanished-workspace-close")
        let viewModel = try makeViewModel(at: directory, repo: repo)
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

    func testUnresolvedWorkspaceRemovalRestoresWindowAfterFailure() async throws {
        let fixture = try makeWorkspaceRemovalFixture()
        defer { fixture.window.close() }

        await fixture.manager.withWorkspaceRemoval(fixture.workspace, from: fixture.source) {
            XCTAssertTrue(fixture.target.isShuttingDown, "an unresolved row must still quiesce its open workspace window")
            return false
        }

        XCTAssertFalse(fixture.target.isShuttingDown, "a failed forget must restore the existing window model")
        XCTAssertTrue(fixture.window.isVisible)
    }

    func testWorkspaceRemovalClosesCapturedWindowAfterPathMoves() async throws {
        let fixture = try makeWorkspaceRemovalFixture()
        defer { fixture.window.close() }

        await fixture.manager.withWorkspaceRemoval(fixture.workspace, from: fixture.source) {
            do {
                try FileManager.default.moveItem(at: fixture.checkout, to: fixture.movedCheckout)
                return true
            } catch {
                XCTFail("move target workspace: \(error)")
                return false
            }
        }

        XCTAssertFalse(fixture.window.isVisible, "success must close the captured window after its path moves")
    }

    func testNormalWindowCloseDoesNotRetainTheViewModelForRepoWork() async throws {
        let manager = try makeManager()
        let (directory, repo) = try makeRepository(named: "normal-window-close")
        var viewModel: RepoViewModel? = try makeViewModel(at: directory, repo: repo)
        let releasedViewModel = { [weak viewModel] in viewModel }
        XCTAssertTrue(try manager.register(XCTUnwrap(viewModel)))
        let completed = LockedFlag()
        viewModel?.runRepoTask { _ in
            completed.setAfterBlocking(seconds: 0.2)
        } onSuccess: { _, _ in }

        manager.repoWindowWillClose(at: directory.path)
        viewModel = nil

        XCTAssertNil(releasedViewModel(), "background repo work retained a closed window's model")
        try await Task.sleep(for: .milliseconds(300))
        XCTAssertTrue(completed.isSet)
    }

    func testCloseRepoWindowNormalizesRepresentedPathAliases() throws {
        let manager = try makeManager()
        let root = try makeTemporaryDirectory(named: "window-alias")
        let directory = root.appending(path: "target")
        let alias = root.appending(path: "alias")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try FileManager.default.createSymbolicLink(at: alias, withDestinationURL: directory)
        let window = makeWindow(representing: alias)
        defer { window.close() }

        manager.closeRepoWindow(at: directory.path)

        XCTAssertFalse(window.isVisible)
    }
}
