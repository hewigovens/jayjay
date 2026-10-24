import JayJayCore
import SwiftUI

struct CloneRepositoryWindowScene: Scene {
    let settings: AppSettings
    let windowManager: RepoWindowManager

    var body: some Scene {
        Window(Localized.Repo.cloneRepository, id: AppWindows.cloneRepository) {
            CloneRepositoryView(recentRepositories: settings.recentRepos) { windowManager.openRepo($0) }
                .id(windowManager.cloneRepositoryGeneration)
                .appEnvironment(settings)
        }
        .handlesExternalEvents(matching: [])
        .windowResizability(.contentSize)
        .restorationBehavior(.disabled)
    }
}
