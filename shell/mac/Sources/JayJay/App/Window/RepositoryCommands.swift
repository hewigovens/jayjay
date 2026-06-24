import JayJayCore
import SwiftUI

struct RepositoryCommands: Commands {
    private var tracker: ActiveRepoTracker {
        .shared
    }

    private var repoPath: String? {
        tracker.repoPath
    }

    private var settings: AppSettings? {
        tracker.settings
    }

    var body: some Commands {
        CommandMenu("Repository") {
            Button { tracker.handler?.showCommandPalette() } label: {
                Label("Command Palette", systemImage: "command")
            }
            .keyboardShortcut("p", modifiers: [.command, .shift])
            .disabled(repoPath == nil)

            Button { tracker.handler?.showUndo() } label: {
                Label("Undo Last Operation", systemImage: "arrow.uturn.backward.circle")
            }
            .keyboardShortcut("u", modifiers: [.command, .shift])
            .disabled(repoPath == nil)

            Divider()

            Button { tracker.handler?.showBookmarkManager() } label: {
                Label("Bookmark Manager", systemImage: "bookmark")
            }
            .keyboardShortcut("b", modifiers: [.command, .shift])
            .disabled(repoPath == nil)

            Button { tracker.handler?.showNewWorkspace() } label: {
                Label("New Workspace...", systemImage: "plus.rectangle.on.folder")
            }
            .disabled(repoPath == nil)

            Divider()

            Button {
                guard let repoPath else { return }
                Self.openRemoteRepository(at: repoPath)
            } label: {
                Label("View Remote Repository", systemImage: "globe")
            }
            .disabled(repoPath == nil)

            Button {
                guard let repoPath else { return }
                RepositoryActions.showInFinder(repoPath: repoPath)
            } label: {
                Label("Show in Finder", systemImage: "folder")
            }
            .keyboardShortcut("f", modifiers: [.command, .option])
            .disabled(repoPath == nil)

            if let settings {
                Button {
                    guard let repoPath else { return }
                    settings.openInEditor(filePath: ".", repoPath: repoPath)
                } label: {
                    Label("Open in \(settings.externalEditor.title)", systemImage: "curlybraces")
                }
                .disabled(repoPath == nil)

                Button {
                    guard let repoPath else { return }
                    settings.openInTerminal(at: repoPath)
                } label: {
                    Label("Open in \(settings.terminal.title)", systemImage: "terminal")
                }
                .disabled(repoPath == nil)
            }
        }
    }

    /// Open the origin remote's web page. Core normalizes ssh/scp remotes to https so we
    /// never hand ssh:// to the system; the blocking open+resolve runs off the main thread.
    static func openRemoteRepository(at path: String) {
        Task.detached {
            guard let repo = try? JayJayRepo.open(path: path),
                  let url = repo.remoteWebUrl().flatMap(URL.init(string:))
            else { return }
            await MainActor.run {
                _ = NSWorkspace.shared.open(url)
            }
        }
    }

    /// Variant for call sites holding a live repo, avoiding a redundant `JayJayRepo.open`.
    @MainActor
    static func openRemoteRepository(repo: JayJayRepo) {
        Task.detached { [repo] in
            guard let url = repo.remoteWebUrl().flatMap(URL.init(string:)) else { return }
            await MainActor.run {
                _ = NSWorkspace.shared.open(url)
            }
        }
    }
}
