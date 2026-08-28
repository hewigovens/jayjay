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
        let settings = AppSettings(defaults: defaults)
        settings.hasCompletedOnboarding = true
        let manager = RepoWindowManager(settings: settings)
        manager.setWindowActions(
            presenting: AppWindows.repo,
            openWindow: { id, value in
                if id == AppWindows.repo, let value {
                    openedPaths(value)
                }
            },
            dismissWindow: { _ in }
        )
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

    func testAppTerminationQuiescesEveryRegisteredRepository() throws {
        let manager = try makeManager()
        let first = try makeRepository(named: "app-shutdown-first")
        let second = try makeRepository(named: "app-shutdown-second")
        let firstViewModel = try makeViewModel(at: first.0, repo: first.1)
        let secondViewModel = try makeViewModel(at: second.0, repo: second.1)
        XCTAssertTrue(manager.register(firstViewModel))
        XCTAssertTrue(manager.register(secondViewModel))

        manager.prepareForTermination()

        XCTAssertTrue(firstViewModel.isShuttingDown)
        XCTAssertTrue(secondViewModel.isShuttingDown)
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

    func testOnboardingDefersRepositoryOpensUntilItFinishes() throws {
        _ = NSApplication.shared
        let suiteName = "RepoWindowManagerTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suiteName))
        addTeardownBlock { defaults.removePersistentDomain(forName: suiteName) }
        let settings = AppSettings(defaults: defaults)
        settings.hasCompletedOnboarding = false
        let manager = RepoWindowManager(settings: settings)
        var opened: [String] = []
        var dismissed: [String] = []
        manager.setWindowActions(
            presenting: AppWindows.onboarding,
            openWindow: { id, value in opened.append(value.map { "\(id):\($0)" } ?? id) },
            dismissWindow: { dismissed.append($0) }
        )
        let repo = try makeTemporaryDirectory(named: "onboarding")
        let restored = makeWindow(representing: repo)

        manager.openRepo(repo.path)
        manager.showRepoList()
        XCTAssertEqual(opened, [AppWindows.onboarding, AppWindows.onboarding])
        XCTAssertFalse(restored.isKeyWindow, "a restored repository window was activated ahead of onboarding")

        restored.close()
        manager.finishOnboarding()

        XCTAssertTrue(settings.hasCompletedOnboarding)
        XCTAssertEqual(opened.count, 3)
        XCTAssertTrue(
            opened[2].hasPrefix("\(AppWindows.repo):") && opened[2].hasSuffix(repo.lastPathComponent),
            "finishing onboarding did not open the deferred repository: \(opened)"
        )
        XCTAssertEqual(dismissed, [AppWindows.onboarding])
    }

    func testLaunchSceneIsAppliedWhenTheWrongSceneCameUp() throws {
        let (manager, repo) = try makeLaunchRoutedManager()
        var opened: [String] = []
        var dismissed: [String] = []
        let applied = expectation(description: "launch scene applied")
        manager.setWindowActions(
            presenting: AppWindows.repoList,
            openWindow: { id, value in
                opened.append(value.map { "\(id):\($0)" } ?? id)
                applied.fulfill()
            },
            dismissWindow: { dismissed.append($0) }
        )

        wait(for: [applied], timeout: 1)

        XCTAssertEqual(opened.count, 1)
        XCTAssertTrue(opened[0].hasPrefix("\(AppWindows.repo):") && opened[0].hasSuffix(repo.lastPathComponent), "\(opened)")
        XCTAssertEqual(dismissed, [AppWindows.onboarding, AppWindows.repoList])
        XCTAssertNil(manager.launchScene)
    }

    func testLaunchSceneIsConfirmedWhenTheRoutedSceneCameUp() throws {
        let (manager, _) = try makeLaunchRoutedManager()
        var opened: [String] = []
        manager.setWindowActions(
            presenting: AppWindows.repo,
            openWindow: { id, value in opened.append(value.map { "\(id):\($0)" } ?? id) },
            dismissWindow: { _ in }
        )
        RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.2))

        XCTAssertTrue(opened.isEmpty, "the routed window was duplicated: \(opened)")
        XCTAssertNil(manager.launchScene)
    }

    func testOnboardingRouteClosesRepositoryWindows() throws {
        let (manager, _) = try makeLaunchRoutedManager(onboarded: false)
        manager.launchScene = .onboarding(nextRepo: nil)
        var opened: [String] = []
        var dismissed: [String] = []
        let applied = expectation(description: "onboarding presented")
        manager.setWindowActions(
            presenting: AppWindows.repo,
            openWindow: { id, _ in
                opened.append(id)
                applied.fulfill()
            },
            dismissWindow: { dismissed.append($0) }
        )

        wait(for: [applied], timeout: 1)

        XCTAssertEqual(opened, [AppWindows.onboarding])
        XCTAssertEqual(dismissed, [AppWindows.repoList, AppWindows.repo])
    }

    func testEmptyRepositoryWindowIsDismissedWhenNothingIsRouted() throws {
        let (manager, _) = try makeLaunchRoutedManager()
        manager.launchScene = nil
        var opened: [String] = []
        var dismissed: [String] = []
        let done = expectation(description: "empty window dismissed")
        manager.setWindowActions(
            presenting: AppWindows.repo,
            openWindow: { id, _ in opened.append(id) },
            dismissWindow: {
                dismissed.append($0)
                done.fulfill()
            }
        )
        manager.emptyRepoWindowDidAppear()

        wait(for: [done], timeout: 1)

        XCTAssertTrue(opened.isEmpty, "\(opened)")
        XCTAssertEqual(dismissed, [AppWindows.repo])
    }

    private func makeLaunchRoutedManager(onboarded: Bool = true) throws -> (RepoWindowManager, URL) {
        _ = NSApplication.shared
        let suiteName = "RepoWindowManagerTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suiteName))
        addTeardownBlock { defaults.removePersistentDomain(forName: suiteName) }
        let settings = AppSettings(defaults: defaults)
        settings.hasCompletedOnboarding = onboarded
        let manager = RepoWindowManager(settings: settings)
        let repo = try makeTemporaryDirectory(named: "launch")
        manager.launchScene = .repo(repo.path)
        return (manager, repo)
    }

    func testClosedWindowIsNotReactivatedForItsRepository() throws {
        var opened: [String] = []
        let manager = try makeManager(openedPaths: { opened.append($0) })
        let repo = try makeTemporaryDirectory(named: "closed")
        let window = makeWindow(representing: repo)
        window.orderFront(nil)
        manager.openRepo(repo.path)
        XCTAssertTrue(opened.isEmpty, "a visible window for the path must be activated, not duplicated")

        window.close()
        manager.openRepo(repo.path)

        XCTAssertEqual(opened.count, 1, "closing the window must make openRepo open a fresh one")
        XCTAssertFalse(window.isVisible, "the closed window was resurrected")
    }
}
