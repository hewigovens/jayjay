import JayJayCore
import SwiftUI

struct HelpCommands: Commands {
    @Environment(\.openWindow) private var openWindow

    var body: some Commands {
        CommandGroup(replacing: .help) {
            Button {
                HelpBook.open()
            } label: {
                Label(Localized.Menu.jayjayHelp, systemImage: "questionmark.circle")
            }
            Link(destination: HelpBook.onlineGuideURL) {
                Label(Localized.Menu.jayjayUserGuide, systemImage: "book")
            }
            Divider()
            Link(destination: AppMetadata.jjDocsURL) {
                Label(Localized.Menu.jujutsuDocumentation, systemImage: "book.closed")
            }
            Divider()
            Link(destination: URL(string: "https://github.com/hewigovens/jayjay/issues")!) {
                Label(Localized.Menu.reportAnIssue, systemImage: "exclamationmark.bubble")
            }
            Button {
                FeedbackEmail.open()
            } label: {
                Label(Localized.Menu.sendFeedback, systemImage: "envelope")
            }
            Button {
                guard let path = logFilePath() else { return }
                NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: path)])
            } label: {
                Label(Localized.Menu.revealLogs, systemImage: "doc.text.magnifyingglass")
            }
            Divider()
            Button { openWindow(id: AppWindows.shortcuts) } label: {
                Label(Localized.Menu.keyboardShortcuts, systemImage: "keyboard")
            }
            .keyboardShortcut(AppShortcut.keyboardShortcuts)
        }
    }
}
