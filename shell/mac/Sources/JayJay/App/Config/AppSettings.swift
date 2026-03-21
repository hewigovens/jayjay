import Foundation

@Observable
final class AppSettings {
    private enum StorageKeys {
        static let fontFamily = "jayjay.fontFamily"
        static let fontSize = "jayjay.fontSize"
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

    // MARK: - Font

    var fontFamily: MonoFont {
        didSet { defaults.set(fontFamily.rawValue, forKey: StorageKeys.fontFamily) }
    }

    var fontSize: Double {
        didSet { defaults.set(fontSize, forKey: StorageKeys.fontSize) }
    }

    // MARK: - Appearance

    var appearanceMode: AppearanceMode {
        didSet { defaults.set(
            appearanceMode.rawValue,
            forKey: StorageKeys.appearanceMode
        ) }
    }

    var diffTheme: DiffTheme {
        didSet { defaults.set(diffTheme.rawValue, forKey: StorageKeys.diffTheme) }
    }

    // MARK: - Diff

    var sideBySideDiff: Bool {
        didSet { defaults.set(sideBySideDiff, forKey: StorageKeys.sideBySideDiff) }
    }

    var ignoreWhitespace: Bool {
        didSet { defaults.set(ignoreWhitespace, forKey: StorageKeys.ignoreWhitespace) }
    }

    var treeFileList: Bool {
        didSet { defaults.set(treeFileList, forKey: StorageKeys.treeFileList) }
    }

    var skipAbandonConfirmation: Bool {
        didSet { defaults.set(
            skipAbandonConfirmation,
            forKey: StorageKeys.skipAbandonConfirmation
        ) }
    }

    // MARK: - Layout

    var sidebarWidth: Double {
        didSet { defaults.set(sidebarWidth, forKey: StorageKeys.sidebarWidth) }
    }

    // MARK: - Repos

    var recentRepos: [String] {
        didSet { defaults.set(recentRepos, forKey: StorageKeys.recentRepos) }
    }

    var lastOpenedRepo: String? {
        didSet { defaults.set(lastOpenedRepo, forKey: StorageKeys.lastOpenedRepo) }
    }

    var hasCompletedOnboarding: Bool {
        didSet { defaults.set(
            hasCompletedOnboarding,
            forKey: StorageKeys.hasCompletedOnboarding
        ) }
    }

    // MARK: - Tools

    var externalEditor: ExternalEditor {
        didSet { defaults.set(
            externalEditor.rawValue,
            forKey: StorageKeys.externalEditor
        ) }
    }

    var customEditorCommand: String {
        didSet { defaults.set(customEditorCommand, forKey: StorageKeys.customEditorCommand) }
    }

    var terminal: Terminal {
        didSet { defaults.set(terminal.rawValue, forKey: StorageKeys.terminal) }
    }

    var customTerminalCommand: String {
        didSet { defaults.set(
            customTerminalCommand,
            forKey: StorageKeys.customTerminalCommand
        ) }
    }

    // MARK: - Init

    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults

        fontFamily = MonoFont(rawValue: defaults.string(forKey: StorageKeys.fontFamily) ?? "") ?? .system
        fontSize = min(max(defaults.object(forKey: StorageKeys.fontSize) as? Double ?? 12.0, 9), 24)
        appearanceMode = AppearanceMode(rawValue: defaults.string(forKey: StorageKeys.appearanceMode) ?? "") ?? .system
        diffTheme = DiffTheme(rawValue: defaults.string(forKey: StorageKeys.diffTheme) ?? "") ?? .auto
        sideBySideDiff = defaults.bool(forKey: StorageKeys.sideBySideDiff)
        ignoreWhitespace = defaults.bool(forKey: StorageKeys.ignoreWhitespace)
        treeFileList = defaults.bool(forKey: StorageKeys.treeFileList)
        skipAbandonConfirmation = defaults.bool(forKey: StorageKeys.skipAbandonConfirmation)
        sidebarWidth = min(max(defaults.object(forKey: StorageKeys.sidebarWidth) as? Double ?? 300, 240), 600)
        recentRepos = (defaults.stringArray(forKey: StorageKeys.recentRepos) ?? []).filter { !$0.isEmpty }
        lastOpenedRepo = defaults.string(forKey: StorageKeys.lastOpenedRepo)
        hasCompletedOnboarding = defaults.bool(forKey: StorageKeys.hasCompletedOnboarding)
        externalEditor = ExternalEditor(rawValue: defaults.string(forKey: StorageKeys.externalEditor) ?? "") ?? .vscode
        customEditorCommand = defaults.string(forKey: StorageKeys.customEditorCommand) ?? ""
        terminal = Terminal(rawValue: defaults.string(forKey: StorageKeys.terminal) ?? "") ?? .terminal
        customTerminalCommand = defaults.string(forKey: StorageKeys.customTerminalCommand) ?? ""
    }

    // MARK: - Repo helpers

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
}
