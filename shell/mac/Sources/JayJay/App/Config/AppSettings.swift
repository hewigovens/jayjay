import Foundation
import JayJayCore

@Observable
final class AppSettings {
    /// UserDefaults key for the anonymous build and OS statistics preference.
    static let sendsAnonymousStatsKey = "jayjay.sendsAnonymousStats"
    static let updateChannelKey = "jayjay.updateChannel"

    private enum StorageKeys {
        static let fontFamily = "jayjay.fontFamily"
        static let fontSize = "jayjay.fontSize"
        static let appearanceMode = "jayjay.appearanceMode"

        static let sideBySideDiff = "jayjay.sideBySideDiff"
        static let ignoreWhitespace = "jayjay.ignoreWhitespace"
        static let hideGitLfsDiffs = "jayjay.hideGitLfsDiffs"
        static let enableGitSubmoduleSupport = "jayjay.showGitSubmoduleChanges"
        static let treeFileList = "jayjay.treeFileList"
        static let sidebarWidth = "jayjay.sidebarWidth"
        static let fileColumnWidth = "jayjay.fileColumnWidth"
        static let recentRepos = "jayjay.recentRepos"
        static let lastOpenedRepo = "jayjay.lastOpenedRepo"
        static let hasCompletedOnboarding = "jayjay.hasCompletedOnboarding"
        static let skipAbandonConfirmation = "jayjay.skipAbandonConfirmation"
        static let confirmDragRebase = "jayjay.confirmDragRebase"
        static let externalEditor = "jayjay.externalEditor"
        static let customEditorCommand = "jayjay.customEditorCommand"
        static let terminal = "jayjay.terminal"
        static let customTerminalCommand = "jayjay.customTerminalCommand"
        static let sponsorActionCount = "jayjay.sponsorActionCount"
        static let sponsorDismissed = "jayjay.sponsorDismissed"
        static let sponsorNextPromptCount = "jayjay.sponsorNextPromptCount"
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

    // MARK: - Diff

    var sideBySideDiff: Bool {
        didSet { defaults.set(sideBySideDiff, forKey: StorageKeys.sideBySideDiff) }
    }

    var ignoreWhitespace: Bool {
        didSet { defaults.set(ignoreWhitespace, forKey: StorageKeys.ignoreWhitespace) }
    }

    var hideGitLfsDiffs: Bool {
        didSet { defaults.set(hideGitLfsDiffs, forKey: StorageKeys.hideGitLfsDiffs) }
    }

    var enableGitSubmoduleSupport: Bool {
        didSet { defaults.set(enableGitSubmoduleSupport, forKey: StorageKeys.enableGitSubmoduleSupport) }
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

    var confirmDragRebase: Bool {
        didSet { defaults.set(confirmDragRebase, forKey: StorageKeys.confirmDragRebase) }
    }

    // MARK: - Layout

    var sidebarWidth: Double {
        didSet { defaults.set(sidebarWidth, forKey: StorageKeys.sidebarWidth) }
    }

    var fileColumnWidth: Double {
        didSet { defaults.set(fileColumnWidth, forKey: StorageKeys.fileColumnWidth) }
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

    // MARK: - Sponsorship

    var sponsorActionCount: Int {
        didSet { defaults.set(sponsorActionCount, forKey: StorageKeys.sponsorActionCount) }
    }

    var sponsorDismissed: Bool {
        didSet { defaults.set(sponsorDismissed, forKey: StorageKeys.sponsorDismissed) }
    }

    var sponsorNextPromptCount: Int {
        didSet { defaults.set(sponsorNextPromptCount, forKey: StorageKeys.sponsorNextPromptCount) }
    }

    // MARK: - Privacy

    /// Share anonymous build and OS statistics without repository, file, or command data.
    var sendsAnonymousStats: Bool {
        didSet { defaults.set(sendsAnonymousStats, forKey: Self.sendsAnonymousStatsKey) }
    }

    // MARK: - Updates

    var updateChannel: UpdateChannel {
        didSet { defaults.set(updateChannelIdentifier(channel: updateChannel), forKey: Self.updateChannelKey) }
    }

    // MARK: - Init

    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults

        fontFamily = MonoFont(rawValue: defaults.string(forKey: StorageKeys.fontFamily) ?? "") ?? .system
        fontSize = min(max(defaults.object(forKey: StorageKeys.fontSize) as? Double ?? 12.0, 9), 24)
        appearanceMode = AppearanceMode(rawValue: defaults.string(forKey: StorageKeys.appearanceMode) ?? "") ?? .system
        sideBySideDiff = defaults.bool(forKey: StorageKeys.sideBySideDiff)
        ignoreWhitespace = defaults.bool(forKey: StorageKeys.ignoreWhitespace)
        hideGitLfsDiffs = defaults.object(forKey: StorageKeys.hideGitLfsDiffs) as? Bool ?? true
        enableGitSubmoduleSupport = defaults.object(forKey: StorageKeys.enableGitSubmoduleSupport) as? Bool ?? false
        treeFileList = defaults.bool(forKey: StorageKeys.treeFileList)
        skipAbandonConfirmation = defaults.bool(forKey: StorageKeys.skipAbandonConfirmation)
        confirmDragRebase = defaults.object(forKey: StorageKeys.confirmDragRebase) as? Bool ?? true
        sidebarWidth = min(max(defaults.object(forKey: StorageKeys.sidebarWidth) as? Double ?? 360, PaneLayout.sidebar.lowerBound), PaneLayout.sidebar.upperBound)
        fileColumnWidth = min(max(defaults.object(forKey: StorageKeys.fileColumnWidth) as? Double ?? 260, PaneLayout.fileColumn.lowerBound), PaneLayout.fileColumn.upperBound)
        var seenRecentRepos = Set<String>()
        recentRepos = (defaults.stringArray(forKey: StorageKeys.recentRepos) ?? [])
            .filter { !$0.isEmpty }
            .map(Self.standardizedRepositoryPath)
            .filter { seenRecentRepos.insert($0).inserted }
        lastOpenedRepo = defaults.string(forKey: StorageKeys.lastOpenedRepo)
            .flatMap { $0.isEmpty ? nil : Self.standardizedRepositoryPath($0) }
        hasCompletedOnboarding = defaults.bool(forKey: StorageKeys.hasCompletedOnboarding)
        externalEditor = ExternalEditor(rawValue: defaults.string(forKey: StorageKeys.externalEditor) ?? "") ?? .vscode
        customEditorCommand = defaults.string(forKey: StorageKeys.customEditorCommand) ?? ""
        terminal = Terminal(rawValue: defaults.string(forKey: StorageKeys.terminal) ?? "") ?? .terminal
        customTerminalCommand = defaults.string(forKey: StorageKeys.customTerminalCommand) ?? ""
        sponsorActionCount = defaults.integer(forKey: StorageKeys.sponsorActionCount)
        sponsorDismissed = defaults.bool(forKey: StorageKeys.sponsorDismissed)
        sponsorNextPromptCount = max(defaults.integer(forKey: StorageKeys.sponsorNextPromptCount), 5)
        sendsAnonymousStats = defaults.object(forKey: Self.sendsAnonymousStatsKey) as? Bool ?? true
        updateChannel = parseUpdateChannel(value: defaults.string(forKey: Self.updateChannelKey) ?? "")
    }

    // MARK: - Repo helpers

    func recordOpenedRepo(_ path: String) {
        let standardizedPath = Self.standardizedRepositoryPath(path)
        recentRepos.removeAll(where: { $0 == standardizedPath })
        recentRepos.insert(standardizedPath, at: 0)
        recentRepos = Array(recentRepos.prefix(12))
        lastOpenedRepo = standardizedPath
    }

    func removeRecentRepo(_ path: String) {
        recentRepos.removeAll(where: { $0 == path })
        if lastOpenedRepo == path {
            lastOpenedRepo = recentRepos.first
        }
    }

    private static func standardizedRepositoryPath(_ path: String) -> String {
        URL(fileURLWithPath: path).standardizedFileURL.path
    }
}
