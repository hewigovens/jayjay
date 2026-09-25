import SwiftUI

struct OverviewWindowScene: Scene {
    let settings: AppSettings
    let windowManager: RepoWindowManager

    var body: some Scene {
        WindowGroup("Repo Overview", id: AppWindows.overview, for: String.self) { $repoPath in
            if !repoPath.isEmpty {
                OverviewWindow(repoPath: repoPath)
                    .environment(windowManager)
                    .appEnvironment(settings)
            }
        } defaultValue: {
            ""
        }
        .handlesExternalEvents(matching: [])
        .defaultSize(WindowFrameStore.defaultSize(key: AppWindows.overview, fallback: CGSize(width: 1280, height: 800)))
        .windowResizability(.contentMinSize)
        .windowToolbarStyle(.unified)
        .restorationBehavior(.disabled)
    }
}
