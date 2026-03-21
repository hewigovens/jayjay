import SwiftUI

@main
struct JayJayApp: App {
    @NSApplicationDelegateAdaptor private var appDelegate: AppDelegate
    @State private var repoPath: String?
    @State private var settings = AppSettings()
    @State private var windowManager: RepoWindowManager

    init() {
        let initialSettings = AppSettings()
        let cliPath = LaunchArguments.repoPath(from: CommandLine.arguments)
        _settings = State(initialValue: initialSettings)
        let wm = RepoWindowManager(settings: initialSettings)
        let initialPath = cliPath ?? initialSettings.lastOpenedRepo
        wm.mainWindowPath = initialPath
        _windowManager = State(initialValue: wm)
        _repoPath = State(initialValue: initialPath)
    }

    var body: some Scene {
        WindowGroup {
            rootContent
                .environment(settings)
                .environment(windowManager)
                .environment(\.jayjayFontScale, settings.fontScale)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
                .onAppear { appDelegate.openHandler = { openRepo(path: $0) } }
        }
        .handlesExternalEvents(matching: [])
        .windowToolbarStyle(.unifiedCompact)
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
        .handlesExternalEvents(matching: [])
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
                    windowManager.mainWindowPath = path
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
            windowManager.mainWindowPath = normalizedPath
        } else {
            windowManager.openRepo(normalizedPath)
        }
    }

}

class AppDelegate: NSObject, NSApplicationDelegate {
    var openHandler: ((String) -> Void)?

    func application(_ application: NSApplication, open urls: [URL]) {
        for url in urls {
            guard url.scheme == URLScheme.scheme, url.host == URLScheme.hostOpen,
                  let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
                  let path = components.queryItems?.first(where: { $0.name == URLScheme.paramPath })?.value
            else { continue }
            openHandler?(path)
        }
    }
}
