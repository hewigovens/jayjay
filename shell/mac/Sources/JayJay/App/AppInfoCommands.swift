import SwiftUI

struct AppInfoCommands: Commands {
    @Environment(\.openWindow) private var openWindow
    @ObservedObject var updater: SparkleUpdater

    var body: some Commands {
        CommandGroup(replacing: .appInfo) {
            Button { openWindow(id: AppWindows.about) } label: {
                Label("About JayJay", systemImage: "info.circle")
            }
            Button { updater.checkForUpdates() } label: {
                Label("Check for Updates...", systemImage: "arrow.down.circle")
            }
            .disabled(!updater.canCheckForUpdates)
            Divider()
            Button { CLIInstaller.installWithFeedback() } label: {
                Label("Install Command Line Tool...", systemImage: "terminal")
            }
        }
    }
}
