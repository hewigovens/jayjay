import JayJayCore
import SwiftUI

struct OpenRemoteRepositoryWindowScene: Scene {
    let settings: AppSettings
    let windowManager: RepoWindowManager

    var body: some Scene {
        Window(Localized.Ssh.openRemoteRepository, id: AppWindows.openRemoteRepository) {
            OpenRemoteRepositoryView { windowManager.openRepo($0) }
                .id(windowManager.openRemoteRepositoryGeneration)
                .appEnvironment(settings)
        }
        .handlesExternalEvents(matching: [])
        .windowResizability(.contentSize)
        .restorationBehavior(.disabled)
    }
}
