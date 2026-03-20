import SwiftUI

struct RepositoryCommands: Commands {
    @FocusedValue(\.jayjayRepoPath) private var repoPath
    @FocusedValue(\.jayjayGitFetch) private var gitFetch
    @FocusedValue(\.jayjayGitPush) private var gitPush
    @FocusedValue(\.jayjayShowUndo) private var showUndo

    var body: some Commands {
        CommandMenu("Repository") {
            Button("Undo (Operation Log)") {
                showUndo?()
            }
            .keyboardShortcut("z")
            .disabled(showUndo == nil)

            Divider()

            Button("Git Fetch") {
                gitFetch?()
            }
            .keyboardShortcut("f", modifiers: [.command, .shift])
            .disabled(gitFetch == nil)

            Button("Git Push") {
                gitPush?()
            }
            .keyboardShortcut("p", modifiers: [.command, .shift])
            .disabled(gitPush == nil)

            Divider()

            Button("Show in Finder") {
                guard let repoPath else { return }
                RepositoryActions.showInFinder(repoPath: repoPath)
            }
            .keyboardShortcut("f", modifiers: [.command, .option])
            .disabled(repoPath == nil)

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
