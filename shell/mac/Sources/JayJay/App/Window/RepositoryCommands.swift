import SwiftUI

struct RepositoryCommands: Commands {
    @FocusedValue(\.jayjayRepoPath) private var focusedRepoPath
    @FocusedValue(\.jayjayGitFetch) private var gitFetch
    @FocusedValue(\.jayjayGitPush) private var gitPush
    @FocusedValue(\.jayjayShowUndo) private var showUndo
    @FocusedValue(\.jayjaySettings) private var settings
    @FocusedValue(\.jayjayCommandPalette) private var commandPalette

    /// Repo path from focused value or from the key window's representedURL
    private var repoPath: String? {
        focusedRepoPath ?? NSApp.keyWindow?.representedURL?.path
    }

    var body: some Commands {
        CommandMenu("Repository") {
            Button("Command Palette") {
                commandPalette?()
            }
            .keyboardShortcut("p", modifiers: [.command, .shift])
            .disabled(commandPalette == nil)

            Button("Undo (Operation Log)") {
                showUndo?()
            }
            .keyboardShortcut("u", modifiers: [.command, .shift])
            .disabled(showUndo == nil)

            Divider()

            Button("View Remote Repository") {
                guard let repoPath else { return }
                if let url = Self.getRemoteURL(at: repoPath) {
                    Self.openGitURL(url)
                }
            }
            .disabled(repoPath == nil)

            Button("Show in Finder") {
                guard let repoPath else { return }
                RepositoryActions.showInFinder(repoPath: repoPath)
            }
            .keyboardShortcut("f", modifiers: [.command, .option])
            .disabled(repoPath == nil)

            if let settings {
                Button("Open in \(settings.externalEditor.title)") {
                    guard let repoPath else { return }
                    settings.openInEditor(filePath: ".", repoPath: repoPath)
                }
                .disabled(repoPath == nil)

                Button("Open in \(settings.terminal.title)") {
                    guard let repoPath else { return }
                    settings.openInTerminal(at: repoPath)
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
