import AppKit
import SwiftUI

@Observable
final class AppSettings {
    enum AppearanceMode: String, CaseIterable, Identifiable {
        case system
        case light
        case dark

        var id: String { rawValue }

        var title: String {
            switch self {
            case .system: "Match System"
            case .light: "Light"
            case .dark: "Dark"
            }
        }

        var colorScheme: ColorScheme? {
            switch self {
            case .system: nil
            case .light: .light
            case .dark: .dark
            }
        }
    }

    enum DiffTheme: String, CaseIterable, Identifiable {
        case auto
        case vsLight = "vs"
        case vsDark = "vs-dark"
        case githubLight = "github-light"
        case githubDark = "github-dark"

        var id: String { rawValue }

        var title: String {
            switch self {
            case .auto: "Match App Theme"
            case .vsLight: "Light"
            case .vsDark: "Dark"
            case .githubLight: "GitHub Light"
            case .githubDark: "GitHub Dark"
            }
        }

        func resolved(for colorScheme: ColorScheme) -> String {
            switch self {
            case .auto: colorScheme == .dark ? "vs-dark" : "vs"
            case .vsLight: "vs"
            case .vsDark: "vs-dark"
            case .githubLight: "github-light"
            case .githubDark: "github-dark"
            }
        }
    }

    enum ExternalEditor: String, CaseIterable, Identifiable {
        case vscode
        case zed
        case vim
        case neovim
        case custom

        var id: String { rawValue }

        var title: String {
            switch self {
            case .vscode: "Visual Studio Code"
            case .zed: "Zed"
            case .vim: "Vim"
            case .neovim: "Neovim"
            case .custom: "Custom"
            }
        }

        var command: String {
            switch self {
            case .vscode: "code"
            case .zed: "zed"
            case .vim: "vim"
            case .neovim: "nvim"
            case .custom: ""
            }
        }

        var bundleId: String? {
            switch self {
            case .vscode: "com.microsoft.VSCode"
            case .zed: "dev.zed.Zed"
            default: nil
            }
        }

        var isInstalled: Bool {
            if let bid = bundleId {
                return NSWorkspace.shared.urlForApplication(withBundleIdentifier: bid) != nil
            }
            return Self.findBinary(command) != nil
        }

        static func findBinary(_ name: String) -> String? {
            guard !name.isEmpty else { return nil }
            let paths = [
                "/opt/homebrew/bin/\(name)",
                "/usr/local/bin/\(name)",
                "\(NSHomeDirectory())/.local/bin/\(name)"
            ]
            return paths.first(where: { FileManager.default.isExecutableFile(atPath: $0) })
        }
    }

    enum Terminal: String, CaseIterable, Identifiable {
        case terminal
        case iterm
        case ghostty
        case custom

        var id: String { rawValue }

        var title: String {
            switch self {
            case .terminal: "Terminal"
            case .iterm: "iTerm2"
            case .ghostty: "Ghostty"
            case .custom: "Custom"
            }
        }

        var bundleId: String {
            switch self {
            case .terminal: "com.apple.Terminal"
            case .iterm: "com.googlecode.iterm2"
            case .ghostty: "com.mitchellh.ghostty"
            case .custom: ""
            }
        }

        var isInstalled: Bool {
            if self == .terminal || self == .custom { return true }
            return NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId) != nil
        }
    }

    private enum StorageKeys {
        static let fontScale = "jayjay.fontScale"
        static let appearanceMode = "jayjay.appearanceMode"
        static let diffTheme = "jayjay.diffTheme"
        static let sideBySideDiff = "jayjay.sideBySideDiff"
        static let ignoreWhitespace = "jayjay.ignoreWhitespace"
        static let treeFileList = "jayjay.treeFileList"
        static let sidebarWidth = "jayjay.sidebarWidth"
        static let recentRepos = "jayjay.recentRepos"
        static let lastOpenedRepo = "jayjay.lastOpenedRepo"
        static let hasCompletedOnboarding = "jayjay.hasCompletedOnboarding"
        static let skipAbandonConfirmation = "jayjay.skipAbandonConfirmation"
        static let externalEditor = "jayjay.externalEditor"
        static let customEditorCommand = "jayjay.customEditorCommand"
        static let terminal = "jayjay.terminal"
        static let customTerminalCommand = "jayjay.customTerminalCommand"
    }

    var fontScale: Double {
        didSet {
            defaults.set(fontScale, forKey: StorageKeys.fontScale)
        }
    }

    var appearanceMode: AppearanceMode {
        didSet {
            defaults.set(appearanceMode.rawValue, forKey: StorageKeys.appearanceMode)
        }
    }

    var diffTheme: DiffTheme {
        didSet {
            defaults.set(diffTheme.rawValue, forKey: StorageKeys.diffTheme)
        }
    }

    var sideBySideDiff: Bool {
        didSet {
            defaults.set(sideBySideDiff, forKey: StorageKeys.sideBySideDiff)
        }
    }

    var ignoreWhitespace: Bool {
        didSet {
            defaults.set(ignoreWhitespace, forKey: StorageKeys.ignoreWhitespace)
        }
    }

    var treeFileList: Bool {
        didSet {
            defaults.set(treeFileList, forKey: StorageKeys.treeFileList)
        }
    }

    var sidebarWidth: Double {
        didSet {
            defaults.set(sidebarWidth, forKey: StorageKeys.sidebarWidth)
        }
    }

    var recentRepos: [String] {
        didSet {
            defaults.set(recentRepos, forKey: StorageKeys.recentRepos)
        }
    }

    var lastOpenedRepo: String? {
        didSet {
            defaults.set(lastOpenedRepo, forKey: StorageKeys.lastOpenedRepo)
        }
    }

    var hasCompletedOnboarding: Bool {
        didSet {
            defaults.set(hasCompletedOnboarding, forKey: StorageKeys.hasCompletedOnboarding)
        }
    }

    var skipAbandonConfirmation: Bool {
        didSet {
            defaults.set(skipAbandonConfirmation, forKey: StorageKeys.skipAbandonConfirmation)
        }
    }

    var externalEditor: ExternalEditor {
        didSet {
            defaults.set(externalEditor.rawValue, forKey: StorageKeys.externalEditor)
        }
    }

    var customEditorCommand: String {
        didSet {
            defaults.set(customEditorCommand, forKey: StorageKeys.customEditorCommand)
        }
    }

    var terminal: Terminal {
        didSet {
            defaults.set(terminal.rawValue, forKey: StorageKeys.terminal)
        }
    }

    var customTerminalCommand: String {
        didSet {
            defaults.set(customTerminalCommand, forKey: StorageKeys.customTerminalCommand)
        }
    }

    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults

        let storedScale = defaults.object(forKey: StorageKeys.fontScale) as? Double ?? 1.0
        self.fontScale = min(max(storedScale, 0.85), 1.45)

        let storedMode = defaults.string(forKey: StorageKeys.appearanceMode)
        self.appearanceMode = AppearanceMode(rawValue: storedMode ?? "") ?? .system

        let storedDiffTheme = defaults.string(forKey: StorageKeys.diffTheme)
        self.diffTheme = DiffTheme(rawValue: storedDiffTheme ?? "") ?? .auto

        self.sideBySideDiff = defaults.bool(forKey: StorageKeys.sideBySideDiff)
        self.ignoreWhitespace = defaults.bool(forKey: StorageKeys.ignoreWhitespace)
        self.treeFileList = defaults.bool(forKey: StorageKeys.treeFileList)

        let storedWidth = defaults.object(forKey: StorageKeys.sidebarWidth) as? Double ?? 300
        self.sidebarWidth = min(max(storedWidth, 240), 600)

        let storedRepos = defaults.stringArray(forKey: StorageKeys.recentRepos) ?? []
        self.recentRepos = storedRepos.filter { !$0.isEmpty }
        self.lastOpenedRepo = defaults.string(forKey: StorageKeys.lastOpenedRepo)
        self.hasCompletedOnboarding = defaults.bool(forKey: StorageKeys.hasCompletedOnboarding)
        self.skipAbandonConfirmation = defaults.bool(forKey: StorageKeys.skipAbandonConfirmation)

        let storedEditor = defaults.string(forKey: StorageKeys.externalEditor)
        self.externalEditor = ExternalEditor(rawValue: storedEditor ?? "") ?? .vscode
        self.customEditorCommand = defaults.string(forKey: StorageKeys.customEditorCommand) ?? ""

        let storedTerminal = defaults.string(forKey: StorageKeys.terminal)
        self.terminal = Terminal(rawValue: storedTerminal ?? "") ?? .terminal
        self.customTerminalCommand = defaults.string(forKey: StorageKeys.customTerminalCommand) ?? ""
    }

    func recordOpenedRepo(_ path: String) {
        let normalizedPath = URL(fileURLWithPath: path).standardizedFileURL.path
        recentRepos.removeAll(where: { $0 == normalizedPath })
        recentRepos.insert(normalizedPath, at: 0)
        recentRepos = Array(recentRepos.prefix(12))
        lastOpenedRepo = normalizedPath
    }

    func removeRecentRepo(_ path: String) {
        recentRepos.removeAll(where: { $0 == path })
        if lastOpenedRepo == path {
            lastOpenedRepo = recentRepos.first
        }
    }

    func openInEditor(filePath: String, repoPath: String) {
        let fullPath = URL(fileURLWithPath: repoPath).appendingPathComponent(filePath).path
        let cmd = externalEditor == .custom ? customEditorCommand : externalEditor.command
        guard !cmd.isEmpty else { return }

        if externalEditor == .vim || externalEditor == .neovim {
            openInTerminal(at: repoPath, command: "\(cmd) \"\(fullPath)\"")
            return
        }

        if let binary = ExternalEditor.findBinary(cmd) {
            let process = Process()
            process.executableURL = URL(fileURLWithPath: binary)
            process.arguments = [fullPath]
            try? process.run()
        }
    }

    func openInTerminal(at path: String, command: String? = nil) {
        let appName = terminal == .custom ? customTerminalCommand : terminal.title
        let cmd = command ?? "cd \"\(path)\""

        if terminal == .terminal || terminal == .custom {
            let script = "tell application \"\(appName)\" to do script \"\(cmd)\""
            if let appleScript = NSAppleScript(source: script) {
                appleScript.executeAndReturnError(nil)
            }
            NSWorkspace.shared.launchApplication(appName)
        } else if let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: terminal.bundleId) {
            let config = NSWorkspace.OpenConfiguration()
            config.arguments = [path]
            NSWorkspace.shared.openApplication(at: url, configuration: config)
        }
    }
}

private struct JayJayFontScaleKey: EnvironmentKey {
    static let defaultValue: Double = 1.0
}

extension EnvironmentValues {
    var jayjayFontScale: Double {
        get { self[JayJayFontScaleKey.self] }
        set { self[JayJayFontScaleKey.self] = newValue }
    }
}

private struct JayJayFontModifier: ViewModifier {
    @Environment(\.jayjayFontScale) private var fontScale

    let size: CGFloat
    let weight: Font.Weight
    let design: Font.Design

    func body(content: Content) -> some View {
        content.font(.system(size: size * fontScale, weight: weight, design: design))
    }
}

extension View {
    func jayjayFont(
        _ size: CGFloat,
        weight: Font.Weight = .regular,
        design: Font.Design = .default
    ) -> some View {
        modifier(JayJayFontModifier(size: size, weight: weight, design: design))
    }
}
