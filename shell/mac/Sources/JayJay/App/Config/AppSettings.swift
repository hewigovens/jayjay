import Foundation
import JayJayCore

@Observable
final class AppSettings {
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
        static let recentRepos = "jayjay.recentRepos"
        static let lastOpenedRepo = "jayjay.lastOpenedRepo"
        static let hasCompletedOnboarding = "jayjay.hasCompletedOnboarding"
        static let savedRevsets = "jayjay.savedRevsets"
        static let evologHideSnapshots = "jayjay.evologHideSnapshots"
        static let evologCollapseSnapshotRuns = "jayjay.evologCollapseSnapshotRuns"
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

    var savedRevsets: [SavedRevset] {
        didSet { saveSavedRevsets() }
    }

    var evologHideSnapshots: Bool {
        didSet { defaults.set(evologHideSnapshots, forKey: StorageKeys.evologHideSnapshots) }
    }

    var evologCollapseSnapshotRuns: Bool {
        didSet {
            defaults.set(
                evologCollapseSnapshotRuns,
                forKey: StorageKeys.evologCollapseSnapshotRuns
            )
        }
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
        sidebarWidth = min(max(defaults.object(forKey: StorageKeys.sidebarWidth) as? Double ?? 360, 240), 600)
        recentRepos = (defaults.stringArray(forKey: StorageKeys.recentRepos) ?? []).filter { !$0.isEmpty }
        lastOpenedRepo = defaults.string(forKey: StorageKeys.lastOpenedRepo)
        hasCompletedOnboarding = defaults.bool(forKey: StorageKeys.hasCompletedOnboarding)
        savedRevsets = Self.loadSavedRevsets(defaults: defaults)
        evologHideSnapshots = defaults.bool(forKey: StorageKeys.evologHideSnapshots)
        evologCollapseSnapshotRuns = defaults.object(
            forKey: StorageKeys.evologCollapseSnapshotRuns
        ) as? Bool ?? true
        externalEditor = ExternalEditor(rawValue: defaults.string(forKey: StorageKeys.externalEditor) ?? "") ?? .vscode
        customEditorCommand = defaults.string(forKey: StorageKeys.customEditorCommand) ?? ""
        terminal = Terminal(rawValue: defaults.string(forKey: StorageKeys.terminal) ?? "") ?? .terminal
        customTerminalCommand = defaults.string(forKey: StorageKeys.customTerminalCommand) ?? ""
        sponsorActionCount = defaults.integer(forKey: StorageKeys.sponsorActionCount)
        sponsorDismissed = defaults.bool(forKey: StorageKeys.sponsorDismissed)
        sponsorNextPromptCount = max(defaults.integer(forKey: StorageKeys.sponsorNextPromptCount), 5)
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

    func saveRevset(name: String, expression: String) {
        savedRevsets = upsertSavedRevset(existing: savedRevsets, name: name, expression: expression)
    }

    func removeSavedRevset(id: String) {
        savedRevsets = JayJayCore.removeSavedRevset(existing: savedRevsets, id: id)
    }

    private static func loadSavedRevsets(defaults: UserDefaults) -> [SavedRevset] {
        if let json = defaults.string(forKey: StorageKeys.savedRevsets) {
            return decodeSavedRevsetsJson(json: json)
        }
        guard let data = defaults.data(forKey: StorageKeys.savedRevsets),
              let json = String(data: data, encoding: .utf8)
        else { return [] }
        return decodeSavedRevsetsJson(json: json)
    }

    private func saveSavedRevsets() {
        defaults.set(encodeSavedRevsetsJson(revsets: savedRevsets), forKey: StorageKeys.savedRevsets)
    }
}
