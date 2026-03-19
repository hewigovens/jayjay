import SwiftUI

@main
struct JayJayApp: App {
    @State private var repoPath: String?
    @State private var settings = AppSettings()
    @State private var windowManager: RepoWindowManager

    init() {
        let initialSettings = AppSettings()
        let cliPath = LaunchArguments.repoPath(from: CommandLine.arguments)
        _settings = State(initialValue: initialSettings)
        _windowManager = State(initialValue: RepoWindowManager(settings: initialSettings))
        _repoPath = State(initialValue: cliPath ?? initialSettings.lastOpenedRepo)
    }

    var body: some Scene {
        WindowGroup {
            rootContent
                .environment(settings)
                .environment(windowManager)
                .environment(\.jayjayFontScale, settings.fontScale)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Open Repository...") {
                    openRepo()
                }
                .keyboardShortcut("o")
            }

            CommandMenu("Recent Repositories") {
                if settings.recentRepos.isEmpty {
                    Button("No Recent Repositories") {}
                        .disabled(true)
                } else {
                    ForEach(settings.recentRepos, id: \.self) { path in
                        Button(path) {
                            openRepo(path: path)
                        }
                    }

                    Divider()

                    Button("Clear Recent Repositories") {
                        settings.recentRepos = []
                        settings.lastOpenedRepo = nil
                    }
                }
            }
        }

        Settings {
            SettingsView()
                .environment(settings)
                .environment(\.jayjayFontScale, settings.fontScale)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
        }
    }

    @ViewBuilder
    private var rootContent: some View {
        if let path = repoPath {
            RepoWindow(repoPath: path)
                .task(id: path) {
                    settings.recordOpenedRepo(path)
                }
        } else {
            WelcomeView(onOpen: { path in
                openRepo(path: path)
            })
        }
    }

    private func openRepo() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        panel.message = "Choose a Jujutsu repository"
        if panel.runModal() == .OK, let url = panel.url {
            openRepo(path: url.path)
        }
    }

    private func openRepo(path: String) {
        let normalizedPath = URL(fileURLWithPath: path).standardizedFileURL.path
        settings.recordOpenedRepo(normalizedPath)
        if repoPath == nil {
            repoPath = normalizedPath
        } else if repoPath == normalizedPath {
            repoPath = normalizedPath
        } else {
            windowManager.openRepo(normalizedPath)
        }
    }
}

private enum LaunchArguments {
    static func repoPath(from arguments: [String]) -> String? {
        var iterator = arguments.dropFirst().makeIterator()

        while let argument = iterator.next() {
            switch argument {
            case "--repo", "-r":
                guard let path = iterator.next() else {
                    return nil
                }
                return normalizedRepoPath(path)
            case let value where value.hasPrefix("--repo="):
                return normalizedRepoPath(String(value.dropFirst("--repo=".count)))
            case "--":
                guard let path = iterator.next() else {
                    return nil
                }
                return normalizedRepoPath(path)
            case let value where value.hasPrefix("-"):
                // Skip the next argument too — it's the value for this flag
                // (e.g. Xcode injects "-NSDocumentRevisionsDebugMode YES")
                _ = iterator.next()
                continue
            default:
                return normalizedRepoPath(argument)
            }
        }

        return nil
    }

    private static func normalizedRepoPath(_ path: String) -> String {
        let url = URL(fileURLWithPath: path, relativeTo: URL(fileURLWithPath: FileManager.default.currentDirectoryPath))
        return url.standardizedFileURL.path
    }
}

struct WelcomeView: View {
    let onOpen: (String) -> Void

    @Environment(AppSettings.self) private var settings

    var body: some View {
        VStack(alignment: .leading, spacing: 22) {
            VStack(alignment: .leading, spacing: 12) {
                Image(systemName: "arrow.triangle.branch")
                    .font(.system(size: 48))
                    .foregroundStyle(.secondary)
                Text("JayJay")
                    .jayjayFont(34, weight: .bold)
                Text("A native GUI for Jujutsu")
                    .jayjayFont(15)
                    .foregroundStyle(.secondary)
                Button("Open Repository...") {
                    let panel = NSOpenPanel()
                    panel.canChooseFiles = false
                    panel.canChooseDirectories = true
                    panel.allowsMultipleSelection = false
                    if panel.runModal() == .OK, let url = panel.url {
                        onOpen(url.path)
                    }
                }
                .keyboardShortcut(.defaultAction)
            }

            if !settings.recentRepos.isEmpty {
                VStack(alignment: .leading, spacing: 10) {
                    HStack {
                        Text("Recent Repositories")
                            .jayjayFont(14, weight: .semibold)
                        Spacer()
                        Button("Clear") {
                            settings.recentRepos = []
                            settings.lastOpenedRepo = nil
                        }
                    }

                    ForEach(settings.recentRepos, id: \.self) { path in
                        HStack(spacing: 8) {
                            Button {
                                onOpen(path)
                            } label: {
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(URL(fileURLWithPath: path).lastPathComponent)
                                        .jayjayFont(13, weight: .medium)
                                    Text(path)
                                        .jayjayFont(11)
                                        .foregroundStyle(.secondary)
                                        .lineLimit(1)
                                        .truncationMode(.middle)
                                }
                                .frame(maxWidth: .infinity, alignment: .leading)
                            }
                            .buttonStyle(.plain)

                            Button {
                                settings.removeRecentRepo(path)
                            } label: {
                                Image(systemName: "xmark.circle.fill")
                                    .foregroundStyle(.tertiary)
                            }
                            .buttonStyle(.plain)
                        }
                        .padding(10)
                        .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
                    }
                }
            }
        }
        .padding(24)
        .frame(minWidth: 460, minHeight: 360, alignment: .topLeading)
    }
}
