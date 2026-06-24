import SwiftUI

struct HelpCommands: Commands {
    @Environment(\.openWindow) private var openWindow

    var body: some Commands {
        CommandGroup(replacing: .help) {
            Button {
                HelpBook.open()
            } label: {
                Label("JayJay Help", systemImage: "questionmark.circle")
            }
            Link(destination: HelpBook.onlineGuideURL) {
                Label("JayJay User Guide", systemImage: "book")
            }
            Divider()
            Link(destination: URL(string: "https://jj-vcs.github.io/jj/latest/")!) {
                Label("Jujutsu Documentation", systemImage: "book.closed")
            }
            Divider()
            Link(destination: URL(string: "https://github.com/hewigovens/jayjay/issues")!) {
                Label("Report an Issue", systemImage: "exclamationmark.bubble")
            }
            Divider()
            Button { openWindow(id: AppWindows.shortcuts) } label: {
                Label("Keyboard Shortcuts", systemImage: "keyboard")
            }
            .keyboardShortcut("/", modifiers: .command)
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
