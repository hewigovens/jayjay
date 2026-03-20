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
            AppInfoCommands()
            RepositoryCommands()

            CommandGroup(replacing: .newItem) {
                Button {
                    openRepo()
                } label: {
                    Label("Open Repository...", systemImage: "folder")
                }
                .keyboardShortcut("o")

                Menu {
                    if settings.recentRepos.isEmpty {
                        Text("No Recent Repositories")
                    } else {
                        ForEach(settings.recentRepos, id: \.self) { path in
                            Button {
                                openRepo(path: path)
                            } label: {
                                Label(
                                    URL(fileURLWithPath: path).lastPathComponent,
                                    systemImage: "arrow.triangle.branch"
                                )
                            }
                        }

                        Divider()

                        Button {
                            settings.recentRepos = []
                            settings.lastOpenedRepo = nil
                        } label: {
                            Label("Clear", systemImage: "trash")
                        }
                    }
                } label: {
                    Label("Open Recent", systemImage: "clock")
                }
            }
        }

        Settings {
            SettingsView()
                .environment(settings)
                .environment(\.jayjayFontScale, settings.fontScale)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
        }

        Window("About JayJay", id: AppWindows.about) {
            AboutView()
                .environment(\.jayjayFontScale, settings.fontScale)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
        }
        .windowResizability(.contentSize)
        .defaultSize(width: 420, height: 460)
    }

    @ViewBuilder
    private var rootContent: some View {
        if !settings.hasCompletedOnboarding {
            OnboardingView {
                settings.hasCompletedOnboarding = true
            }
        } else if let path = repoPath {
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

private struct AppInfoCommands: Commands {
    @Environment(\.openWindow) private var openWindow

    var body: some Commands {
        CommandGroup(replacing: .appInfo) {
            Button("About JayJay") {
                openWindow(id: AppWindows.about)
            }
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
