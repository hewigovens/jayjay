import JayJayCore
import SwiftUI

struct FileHistoryWindowScene: Scene {
    let settings: AppSettings
    let windowManager: RepoWindowManager

    var body: some Scene {
        WindowGroup(Localized.Menu.fileHistory, id: AppWindows.fileHistory, for: String.self) { $value in
            if let request = FileHistoryRequest(windowValue: value) {
                FileHistoryWindow(request: request)
                    .environment(windowManager)
                    .appEnvironment(settings)
            }
        } defaultValue: {
            ""
        }
        .handlesExternalEvents(matching: [])
        .defaultSize(WindowFrameStore.defaultSize(key: AppWindows.fileHistory, fallback: CGSize(width: 1100, height: 700)))
        .windowResizability(.contentMinSize)
        .restorationBehavior(.disabled)
    }
}
