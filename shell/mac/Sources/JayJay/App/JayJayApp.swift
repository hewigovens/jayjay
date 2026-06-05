import SwiftUI

@main
struct JayJayApp: App {
    @NSApplicationDelegateAdaptor private var appDelegate: AppDelegate
    @State private var repoPath: String?
    @State private var settings = AppSettings()
    @State private var windowManager: RepoWindowManager
    private let updater = SparkleUpdater()

    init() {
        NSWindow.allowsAutomaticWindowTabbing = false

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
                .environment(\.jayjayFontSize, settings.fontSize)
                .environment(\.jayjayFontFamily, settings.fontFamily)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
                .onAppear {
                    appDelegate.openRepositoryPicker = { openRepo() }
                    appDelegate.openHandler = { openRepo(path: $0) }
                    appDelegate.showRepoSelector = { repoPath = nil }
                    appDelegate.recentReposProvider = { [settings] in settings.recentRepos }
                    DebugBadge.apply()
                }
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

        Settings {
            SettingsView(updater: updater)
                .environment(settings)
                .environment(\.jayjayFontSize, settings.fontSize)
                .environment(\.jayjayFontFamily, settings.fontFamily)
                .preferredColorScheme(settings.appearanceMode.colorScheme)
        }

        Window("About JayJay", id: AppWindows.about) {
            AboutView()
                .environment(\.jayjayFontSize, settings.fontSize)
                .environment(\.jayjayFontFamily, settings.fontFamily)
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
            .background(WindowContentSizer(targetSize: OnboardingView.preferredSize, minimumOnly: false))
        } else if let path = repoPath {
            RepoWindow(repoPath: path)
                .task(id: path) {
                    settings.recordOpenedRepo(path)
                    windowManager.mainWindowPath = path
                }
                .background(WindowContentSizer(targetSize: NSSize(width: 1100, height: 700), minimumOnly: true))
        } else {
            WelcomeView(onOpen: { path in
                openRepo(path: path)
            })
            .background(WindowContentSizer(targetSize: NSSize(width: 480, height: 600), minimumOnly: false))
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

private let diffTextViewID = NSUserInterfaceItemIdentifier("diffTextView")

private func findDiffTextView(in view: NSView?) -> NSTextView? {
    guard let view else { return nil }
    if let tv = view as? NSTextView, tv.identifier == diffTextViewID { return tv }
    for sub in view.subviews {
        if let found = findDiffTextView(in: sub) { return found }
    }
    return nil
}

class AppDelegate: NSObject, NSApplicationDelegate {
    var openRepositoryPicker: (() -> Void)?
    var openHandler: ((String) -> Void)?
    var showRepoSelector: (() -> Void)?
    var recentReposProvider: (() -> [String])?

    func applicationDockMenu(_ sender: NSApplication) -> NSMenu? {
        let menu = NSMenu()
        let openItem = NSMenuItem(
            title: "Open Repository...",
            action: #selector(dockMenuOpenRepositoryPicker),
            keyEquivalent: ""
        )
        openItem.target = self
        menu.addItem(openItem)

        let repos = recentReposProvider?() ?? []
        guard !repos.isEmpty else { return menu }

        menu.addItem(.separator())
        let submenu = NSMenu(title: "Recent Repositories")
        for path in repos {
            let name = URL(fileURLWithPath: path).lastPathComponent
            let item = NSMenuItem(title: name, action: #selector(dockMenuOpenRepo(_:)), keyEquivalent: "")
            item.target = self
            item.representedObject = path
            submenu.addItem(item)
        }
        let recentItem = NSMenuItem(title: "Recent Repositories", action: nil, keyEquivalent: "")
        recentItem.submenu = submenu
        menu.addItem(recentItem)
        return menu
    }

    @objc private func dockMenuOpenRepositoryPicker() {
        openRepositoryPicker?()
    }

    @objc private func dockMenuOpenRepo(_ sender: NSMenuItem) {
        guard let path = sender.representedObject as? String else { return }
        openHandler?(path)
    }

    func application(_ application: NSApplication, open urls: [URL]) {
        for url in urls {
            guard url.scheme == URLScheme.scheme, url.host == URLScheme.hostOpen,
                  let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
                  let path = components.queryItems?.first(where: { $0.name == URLScheme.paramPath })?.value
            else { continue }
            openHandler?(path)
        }
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag {
            showRepoSelector?()
        }
        return true
    }
}
