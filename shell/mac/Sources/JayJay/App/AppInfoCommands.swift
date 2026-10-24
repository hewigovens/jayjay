import JayJayCore
import SwiftUI

struct AppInfoCommands: Commands {
    @Environment(\.openWindow) private var openWindow
    @ObservedObject var updater: SparkleUpdater

    var body: some Commands {
        CommandGroup(replacing: .appInfo) {
            Button { openWindow(id: AppWindows.about) } label: {
                Label(Localized.Menu.aboutJayjay, systemImage: "info.circle")
            }
            Button { updater.checkForUpdates() } label: {
                Label(Localized.Menu.checkForUpdatesEllipsis, systemImage: "arrow.down.circle")
            }
            .disabled(!updater.canCheckForUpdates)
            Divider()
            Button { CLIInstaller.installWithFeedback() } label: {
                Label(Localized.Menu.installCommandLineToolEllipsis, systemImage: "terminal")
            }
        }
    }
}
