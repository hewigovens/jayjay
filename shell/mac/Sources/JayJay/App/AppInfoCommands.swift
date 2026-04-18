import SwiftUI

struct HelpCommands: Commands {
    var body: some Commands {
        CommandGroup(replacing: .help) {
            Link(destination: URL(string: "https://github.com/hewigovens/jayjay")!) {
                Label("JayJay on GitHub", systemImage: "arrow.up.right.square")
            }
            Link(destination: URL(string: "https://jj-vcs.github.io/jj/latest/")!) {
                Label("Jujutsu Documentation", systemImage: "book")
            }
            Divider()
            Link(destination: URL(string: "https://github.com/hewigovens/jayjay/issues")!) {
                Label("Report an Issue", systemImage: "exclamationmark.bubble")
            }
        }
    }
}

struct AppInfoCommands: Commands {
    @Environment(\.openWindow) private var openWindow
    let updater: SparkleUpdater

    var body: some Commands {
        CommandGroup(replacing: .appInfo) {
            Button { openWindow(id: AppWindows.about) } label: {
                Label("About JayJay", systemImage: "info.circle")
            }
            Button { updater.checkForUpdates() } label: {
                Label("Check for Updates...", systemImage: "arrow.down.circle")
            }
            Divider()
            Button { CLIInstaller.installWithFeedback() } label: {
                Label("Install Command Line Tool...", systemImage: "terminal")
            }
        }
    }
}
