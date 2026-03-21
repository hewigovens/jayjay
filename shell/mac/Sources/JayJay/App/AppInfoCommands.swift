import SwiftUI

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
