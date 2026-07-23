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
            Button {
                FeedbackEmail.open()
            } label: {
                Label("Send Feedback", systemImage: "envelope")
            }
            Divider()
            Button { openWindow(id: AppWindows.shortcuts) } label: {
                Label("Keyboard Shortcuts", systemImage: "keyboard")
            }
            .keyboardShortcut("/", modifiers: .command)
        }
    }
}
