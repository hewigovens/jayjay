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
                Label("Undo (Operation Log)", systemImage: "arrow.uturn.backward.circle")
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
                if let url = Self.getRemoteURL(at: repoPath) { Self.openGitURL(url) }
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

    private static func getRemoteURL(at path: String) -> String? {
        let proc = Process()
        proc.executableURL = URL(fileURLWithPath: "/usr/bin/git")
        proc.arguments = ["remote", "get-url", "origin"]
        proc.currentDirectoryURL = URL(fileURLWithPath: path)
        let pipe = Pipe()
        proc.standardOutput = pipe
        proc.standardError = FileHandle.nullDevice
        try? proc.run()
        proc.waitUntilExit()
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        let url = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines)
        return url?.isEmpty == true ? nil : url
    }

    private static func openGitURL(_ raw: String) {
        var urlString = raw
        if urlString.hasPrefix("git@") {
            urlString = String(urlString.dropFirst(4))
            if let colonIdx = urlString.firstIndex(of: ":") {
                urlString.replaceSubrange(colonIdx ... colonIdx, with: "/")
            }
            urlString = "https://\(urlString)"
        }
        if urlString.hasSuffix(".git") {
            urlString = String(urlString.dropLast(4))
        }
        if let url = URL(string: urlString) {
            NSWorkspace.shared.open(url)
        }
    }
}
