import SwiftUI

struct RepoWindowGroup: Scene {
    let launchScene: LaunchScene
    let settings: AppSettings
    let repositoryStore: RepositoryStore
    let windowManager: RepoWindowManager
    let updater: SparkleUpdater

    var body: some Scene {
        WindowGroup("JayJay", id: AppWindows.repo, for: String.self) { $repoPath in
            if repoPath.isEmpty {
                // SwiftUI can present this group without a path on a launch routed elsewhere; registering lets the manager route.
                RepoListWindowBridge(windowManager: windowManager, scene: AppWindows.repo, onRegistered: windowManager.emptyRepoWindowDidAppear)
            } else {
                RepoWindowScene(repoPath: repoPath, windowManager: windowManager)
                    .task(id: repoPath) { settings.recordOpenedRepo(repoPath) }
                    .environment(repositoryStore)
                    .environment(windowManager)
                    .appEnvironment(settings)
                    .background(RepoListWindowBridge(windowManager: windowManager, scene: AppWindows.repo))
            }
        } defaultValue: {
            launchScene.repoPath
        }
        .handlesExternalEvents(matching: [])
        .defaultSize(WindowFrameStore.defaultSize(key: AppWindows.repo, fallback: CGSize(width: 1100, height: 700)))
        .windowResizability(.contentMinSize)
        .windowToolbarStyle(.unified)
        .defaultLaunchBehavior(launchScene.repoBehavior)
        .restorationBehavior(.disabled)
        .commands {
            AppInfoCommands(updater: updater)
            RepositoryCommands()
            HelpCommands()
            AppCommands(settings: settings, windowManager: windowManager)
        }
    }
}
