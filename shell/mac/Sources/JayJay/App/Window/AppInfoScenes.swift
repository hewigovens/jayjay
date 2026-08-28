import SwiftUI

struct AppInfoScenes: Scene {
    let settings: AppSettings
    let updater: SparkleUpdater
    let windowManager: RepoWindowManager

    var body: some Scene {
        Settings {
            SettingsView(updater: updater, windowManager: windowManager)
                .appEnvironment(settings)
        }

        Window("About JayJay", id: AppWindows.about) {
            AboutView(updater: updater)
                .appEnvironment(settings)
        }
        .handlesExternalEvents(matching: [])
        .windowResizability(.contentSize)
        .defaultSize(width: 420, height: 460)

        Window("Keyboard Shortcuts", id: AppWindows.shortcuts) {
            KeyboardShortcutsView()
                .appEnvironment(settings)
        }
        .handlesExternalEvents(matching: [])
        .windowStyle(.hiddenTitleBar)
        .windowResizability(.contentSize)
        .defaultSize(width: 720, height: 560)
    }
}
