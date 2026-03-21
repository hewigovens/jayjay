import SwiftUI

struct HelpCommands: Commands {
    var body: some Commands {
        CommandGroup(replacing: .help) {
            Link("JayJay on GitHub", destination: URL(string: "https://github.com/hewigovens/jayjay")!)
            Link("Jujutsu Documentation", destination: URL(string: "https://jj-vcs.github.io/jj/latest/")!)
            Divider()
            Link("Report an Issue", destination: URL(string: "https://github.com/hewigovens/jayjay/issues")!)
        }
    }
}

struct AppInfoCommands: Commands {
    @Environment(\.openWindow) private var openWindow

    var body: some Commands {
        CommandGroup(replacing: .appInfo) {
            Button("About JayJay") {
                openWindow(id: AppWindows.about)
            }
            Divider()
            Button {
                CLIInstaller.install()
            } label: {
                Label("Install CLI...", systemImage: "terminal")
            }
        }
    }
}
