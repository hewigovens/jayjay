import SwiftUI

struct RepositoryCommands: Commands {
    @FocusedValue(\.jayjayRepoPath) private var repoPath

    var body: some Commands {
        CommandMenu("Repository") {
            Button("Show in Finder") {
                guard let repoPath else { return }
                RepositoryActions.showInFinder(repoPath: repoPath)
            }
            .keyboardShortcut("f", modifiers: [.command, .option])
            .disabled(repoPath == nil)

            Divider()

            Button("Open in Visual Studio Code") {
                guard let repoPath else { return }
                RepositoryActions.openInVSCode(repoPath: repoPath)
            }
            .disabled(repoPath == nil || !RepositoryActions.isVSCodeInstalled)

            Button("Open in Ghostty") {
                guard let repoPath else { return }
                RepositoryActions.openInGhostty(repoPath: repoPath)
            }
            .disabled(repoPath == nil || !RepositoryActions.isGhosttyInstalled)
        }
    }
}
