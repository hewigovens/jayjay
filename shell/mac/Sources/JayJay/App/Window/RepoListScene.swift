import SwiftUI

struct RepoListScene: Scene {
    let launchScene: LaunchScene
    let settings: AppSettings
    let repositoryStore: RepositoryStore
    let windowManager: RepoWindowManager

    var body: some Scene {
        Window("JayJay", id: AppWindows.repoList) {
            WelcomeView(onOpen: { windowManager.openRepo($0) })
                .environment(repositoryStore)
                .appEnvironment(settings)
                .background(RepoListWindowBridge(windowManager: windowManager, scene: AppWindows.repoList))
                .background(WindowFramePersistence(key: AppWindows.repoList))
        }
        .handlesExternalEvents(matching: [])
        .defaultSize(WindowFrameStore.defaultSize(key: AppWindows.repoList, fallback: WelcomeView.minimumSize))
        .defaultLaunchBehavior(launchScene.repoListBehavior)
        .restorationBehavior(.disabled)
    }
}
