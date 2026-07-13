import SwiftUI

@main
struct JayJayApp: App {
    @NSApplicationDelegateAdaptor private var appDelegate: AppDelegate
    @State private var repoPath: String?
    @State private var settings = AppSettings()
    @State private var windowManager: RepoWindowManager
    private let updater = SparkleUpdater()

    init() {
        CommandLineInterface.runAndExitIfNeeded(arguments: CommandLine.arguments)

        NSWindow.allowsAutomaticWindowTabbing = false

        let initialSettings = AppSettings()
        let cliPath = LaunchArguments.repoPath(from: CommandLine.arguments)
        _settings = State(initialValue: initialSettings)
        let wm = RepoWindowManager(settings: initialSettings)
        let initialPath = cliPath ?? initialSettings.lastOpenedRepo
        _windowManager = State(initialValue: wm)
        _repoPath = State(initialValue: initialPath)
    }

    var body: some Scene {
        WindowGroup(id: AppWindows.main) {
            rootContent
                .environment(settings)
                .environment(windowManager)
                .environment(\.jayjayFontSize, settings.fontSize)
                .environment(\.jayjayFontFamily, settings.fontFamily)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
                .onAppear {
                    appDelegate.openRepositoryPicker = { openRepo() }
                    appDelegate.openHandler = { openRepo(path: $0) }
                    appDelegate.showRepoSelector = { windowManager.showRepoList() }
                    appDelegate.recentReposProvider = { [settings] in settings.recentRepos }
                }
                .background(RepoListWindowBridge(repoPath: $repoPath, windowManager: windowManager))
        }
        .handlesExternalEvents(matching: [])
        .defaultSize(width: 1100, height: 700)
        .windowResizability(.contentMinSize)
        .windowToolbarStyle(.unified)
        .commands {
            AppInfoCommands(updater: updater)
            RepositoryCommands()
            HelpCommands()

            CommandGroup(replacing: .windowArrangement) {}
            CommandGroup(replacing: .singleWindowList) {}

            CommandGroup(after: .pasteboard) {
                Button {
                    if let window = NSApp.keyWindow,
                       let tv = findDiffTextView(in: window.contentView)
                    {
                        window.makeFirstResponder(tv)
                        let item = NSMenuItem()
                        item.tag = Int(NSFindPanelAction.showFindPanel.rawValue)
                        tv.performFindPanelAction(item)
                    }
                } label: {
                    Label("Find...", systemImage: "magnifyingglass")
                }
                .keyboardShortcut("f")
            }

            CommandGroup(after: .textFormatting) {
                Button { settings.fontSize = min(24, settings.fontSize + 1) } label: {
                    Label("Zoom In", systemImage: "plus.magnifyingglass")
                }
                .keyboardShortcut("+", modifiers: .command)

                Button { settings.fontSize = max(9, settings.fontSize - 1) } label: {
                    Label("Zoom Out", systemImage: "minus.magnifyingglass")
                }
                .keyboardShortcut("-", modifiers: .command)

                Button { settings.fontSize = 12 } label: {
                    Label("Reset Zoom", systemImage: "1.magnifyingglass")
                }
                .keyboardShortcut("0", modifiers: .command)
            }

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

        WindowGroup("JayJay", id: AppWindows.repo, for: String.self) { repoPath in
            repoWindowContent(for: repoPath)
        }
        .handlesExternalEvents(matching: [])
        .defaultSize(width: 1100, height: 700)
        .windowResizability(.contentMinSize)
        .windowToolbarStyle(.unified)

        Settings {
            SettingsView(updater: updater)
                .environment(settings)
                .environment(\.jayjayFontSize, settings.fontSize)
                .environment(\.jayjayFontFamily, settings.fontFamily)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
        }

        Window("About JayJay", id: AppWindows.about) {
            AboutView()
                .environment(settings)
                .environment(\.jayjayFontSize, settings.fontSize)
                .environment(\.jayjayFontFamily, settings.fontFamily)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
        }
        .handlesExternalEvents(matching: [])
        .windowResizability(.contentSize)
        .defaultSize(width: 420, height: 460)

        Window("Keyboard Shortcuts", id: AppWindows.shortcuts) {
            KeyboardShortcutsView()
                .environment(settings)
                .environment(\.jayjayFontSize, settings.fontSize)
                .environment(\.jayjayFontFamily, settings.fontFamily)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
        }
        .handlesExternalEvents(matching: [])
        .windowStyle(.hiddenTitleBar)
        .windowResizability(.contentSize)
        .defaultSize(width: 720, height: 560)
    }

    @ViewBuilder
    private func repoWindowContent(for repoPath: Binding<String?>) -> some View {
        if let path = repoPath.wrappedValue {
            RepoWindowScene(repoPath: path, windowManager: windowManager)
                .environment(settings)
                .environment(windowManager)
                .environment(\.jayjayFontSize, settings.fontSize)
                .environment(\.jayjayFontFamily, settings.fontFamily)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
        }
    }

    @ViewBuilder
    private var rootContent: some View {
        if !settings.hasCompletedOnboarding {
            OnboardingView {
                settings.hasCompletedOnboarding = true
            }
            .background(WindowContentSizer(targetSize: OnboardingView.preferredSize, minimumOnly: false))
        } else if let path = repoPath {
            RepoWindowScene(repoPath: path, windowManager: windowManager)
                .task(id: path) {
                    settings.recordOpenedRepo(path)
                }
                .background(WindowContentSizer(targetSize: NSSize(width: 1100, height: 700), minimumOnly: true))
        } else {
            WelcomeView(onOpen: { path in
                openRepo(path: path)
            })
            .background(WindowContentSizer(targetSize: WelcomeView.minimumSize, minimumOnly: false))
            .background(WindowConfigurator { window in
                window.identifier = NSUserInterfaceItemIdentifier(AppWindows.welcome)
                window.representedURL = nil
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
        windowManager.openRepo(normalizedPath)
    }
}

private let diffTextViewID = NSUserInterfaceItemIdentifier("diffTextView")

private func findDiffTextView(in view: NSView?) -> NSTextView? {
    guard let view else { return nil }
    if let tv = view as? NSTextView, tv.identifier == diffTextViewID {
        return tv
    }
    for sub in view.subviews {
        if let found = findDiffTextView(in: sub) {
            return found
        }
    }
    return nil
}
