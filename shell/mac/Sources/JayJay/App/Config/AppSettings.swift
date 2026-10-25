import Foundation
import JayJayCore

@Observable
final class AppSettings {
    /// UserDefaults key for the anonymous build and OS statistics preference.
    static let sendsAnonymousStatsKey = "jayjay.sendsAnonymousStats"
    static let updateChannelKey = "jayjay.updateChannel"

    private enum StorageKeys {
        static let interfaceFontFamily = "jayjay.interfaceFontFamily"
        static let interfaceFontSize = "jayjay.interfaceFontSize"
        // Pre-split names: the single font setting always drove code text.
        static let codeFontFamily = "jayjay.fontFamily"
        static let codeFontSize = "jayjay.fontSize"
        static let codeLineSpacing = "jayjay.codeLineSpacing"
        static let codeLigatures = "jayjay.codeLigatures"
        static let appearanceMode = "jayjay.appearanceMode"
        static let language = "jayjay.language"
        static let appleLanguages = "AppleLanguages"
        static let diffPaletteLight = "jayjay.diffPaletteLight"
        static let diffPaletteDark = "jayjay.diffPaletteDark"

        static let sideBySideDiff = "jayjay.sideBySideDiff"
        static let ignoreWhitespace = "jayjay.ignoreWhitespace"
        static let hideGitLfsDiffs = "jayjay.hideGitLfsDiffs"
        static let enableGitSubmoduleSupport = "jayjay.showGitSubmoduleChanges"
        static let hideReviewedFiles = "jayjay.hideReviewedFiles"
        static let treeFileList = "jayjay.treeFileList"
        static let autoExpandDescription = "jayjay.autoExpandDescription"
        static let tintWindowWithWallpaper = "jayjay.tintWindowWithWallpaper"
        static let sidebarWidth = "jayjay.sidebarWidth"
        static let sidebarHidden = "jayjay.sidebarHidden"
        static let secondaryPaneWidth = "jayjay.secondaryPaneWidth"
        static let legacyFileColumnWidth = "jayjay.fileColumnWidth"
        static let recentRepos = "jayjay.recentRepos"
        static let lastOpenedRepo = "jayjay.lastOpenedRepo"
        static let hasCompletedOnboarding = "jayjay.hasCompletedOnboarding"
        static let showsRecentRepositoriesPanel = "jayjay.showsRecentRepositoriesPanel"
        static let skipAbandonConfirmation = "jayjay.skipAbandonConfirmation"
        static let skipWorkspaceDeleteConfirmation = "jayjay.skipWorkspaceDeleteConfirmation"
        static let confirmDragRebase = "jayjay.confirmDragRebase"
        static let externalEditor = "jayjay.externalEditor"
        static let customEditorCommand = "jayjay.customEditorCommand"
        static let terminal = "jayjay.terminal"
        static let customTerminalCommand = "jayjay.customTerminalCommand"
        static let aiProviderOrder = "jayjay.aiProviderOrder"
        // Old names: new keys would re-show the prompt to people who turned it off.
        static let ratingPromptActionCount = "jayjay.sponsorActionCount"
        static let ratingPromptDismissed = "jayjay.sponsorDismissed"
        static let ratingPromptNextCount = "jayjay.sponsorNextPromptCount"
    }

    // MARK: - Font

    static let defaultFontSize: Double = 12
    static let fontSizeRange: ClosedRange<Double> = 9 ... 24
    static let lineSpacingRange: ClosedRange<Double> = 1 ... 2

    var interfaceFontFamily: FontFamily {
        didSet { defaults.set(interfaceFontFamily.rawValue, forKey: StorageKeys.interfaceFontFamily) }
    }

    var interfaceFontSize: Double {
        didSet { defaults.set(interfaceFontSize, forKey: StorageKeys.interfaceFontSize) }
    }

    var codeFontFamily: FontFamily {
        didSet { defaults.set(codeFontFamily.rawValue, forKey: StorageKeys.codeFontFamily) }
    }

    var codeFontSize: Double {
        didSet { defaults.set(codeFontSize, forKey: StorageKeys.codeFontSize) }
    }

    var codeLineSpacing: Double {
        didSet { defaults.set(codeLineSpacing, forKey: StorageKeys.codeLineSpacing) }
    }

    var codeLigatures: Bool {
        didSet { defaults.set(codeLigatures, forKey: StorageKeys.codeLigatures) }
    }

    var typography: Typography {
        Typography(
            interfaceFamily: interfaceFontFamily,
            interfaceSize: interfaceFontSize,
            codeFamily: codeFontFamily,
            codeSize: codeFontSize,
            codeLineSpacing: codeLineSpacing,
            codeLigatures: codeLigatures
        )
    }

    func zoomIn() {
        zoom(by: 1)
    }

    func zoomOut() {
        zoom(by: -1)
    }

    private func zoom(by step: Double) {
        interfaceFontSize = (interfaceFontSize + step).clamped(to: Self.fontSizeRange)
        codeFontSize = (codeFontSize + step).clamped(to: Self.fontSizeRange)
    }

    func resetZoom() {
        interfaceFontSize = Self.defaultFontSize
        codeFontSize = Self.defaultFontSize
    }

    func resetFonts() {
        resetZoom()
        interfaceFontFamily = .system
        codeFontFamily = .system
        codeLineSpacing = Self.lineSpacingRange.lowerBound
        codeLigatures = true
    }

    // MARK: - Appearance

    var appearanceMode: AppearanceMode {
        didSet { defaults.set(
            appearanceMode.rawValue,
            forKey: StorageKeys.appearanceMode
        ) }
    }

    /// AppKit and Sparkle keep the language they launched with.
    let launchLanguage: String?

    var language: String? {
        didSet {
            defaults.set(language, forKey: StorageKeys.language)
            // AppKit reads AppleLanguages only at launch, so its own menus and panels follow on the next one.
            if let language {
                defaults.set([language], forKey: StorageKeys.appleLanguages)
            } else {
                defaults.removeObject(forKey: StorageKeys.appleLanguages)
            }
            Localized.setLanguage(language)
        }
    }

    // MARK: - Diff

    var diffPaletteLight: String {
        didSet {
            defaults.set(diffPaletteLight, forKey: StorageKeys.diffPaletteLight)
            lightDiffPalette = diffPalette(id: diffPaletteLight, isDark: false)
        }
    }

    var diffPaletteDark: String {
        didSet {
            defaults.set(diffPaletteDark, forKey: StorageKeys.diffPaletteDark)
            darkDiffPalette = diffPalette(id: diffPaletteDark, isDark: true)
        }
    }

    private(set) var lightDiffPalette: DiffPalette
    private(set) var darkDiffPalette: DiffPalette

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

    var hideReviewedFiles: Bool {
        didSet { defaults.set(hideReviewedFiles, forKey: StorageKeys.hideReviewedFiles) }
    }

    var treeFileList: Bool {
        didSet { defaults.set(treeFileList, forKey: StorageKeys.treeFileList) }
    }

    var autoExpandDescription: Bool {
        didSet { defaults.set(autoExpandDescription, forKey: StorageKeys.autoExpandDescription) }
    }

    var tintWindowWithWallpaper: Bool {
        didSet { defaults.set(tintWindowWithWallpaper, forKey: StorageKeys.tintWindowWithWallpaper) }
    }

    var skipAbandonConfirmation: Bool {
        didSet { defaults.set(
            skipAbandonConfirmation,
            forKey: StorageKeys.skipAbandonConfirmation
        ) }
    }

    var skipWorkspaceDeleteConfirmation: Bool {
        didSet { defaults.set(skipWorkspaceDeleteConfirmation, forKey: StorageKeys.skipWorkspaceDeleteConfirmation) }
    }

    var confirmDragRebase: Bool {
        didSet { defaults.set(confirmDragRebase, forKey: StorageKeys.confirmDragRebase) }
    }

    // MARK: - Layout

    var sidebarWidth: Double {
        didSet { defaults.set(sidebarWidth, forKey: StorageKeys.sidebarWidth) }
    }

    var secondaryPaneWidth: Double {
        didSet { defaults.set(secondaryPaneWidth, forKey: StorageKeys.secondaryPaneWidth) }
    }

    var sidebarHidden: Bool {
        didSet { defaults.set(sidebarHidden, forKey: StorageKeys.sidebarHidden) }
    }

    // MARK: - Repos

    var recentRepos: [String] {
        didSet {
            defaults.set(recentRepos, forKey: StorageKeys.recentRepos)
            // The first recent shows the panel and the last removal hides it, whatever was chosen before.
            if recentRepos.isEmpty != oldValue.isEmpty {
                showsRecentRepositoriesPanel = !recentRepos.isEmpty
            }
        }
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

    var showsRecentRepositoriesPanel: Bool {
        didSet { defaults.set(showsRecentRepositoriesPanel, forKey: StorageKeys.showsRecentRepositoriesPanel) }
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

    var aiProviderOrder: [AiProvider] {
        didSet {
            defaults.set(aiProviderOrder.map { aiProviderId(provider: $0) }, forKey: StorageKeys.aiProviderOrder)
        }
    }

    // MARK: - Rating Prompt

    var ratingPromptActionCount: Int {
        didSet { defaults.set(ratingPromptActionCount, forKey: StorageKeys.ratingPromptActionCount) }
    }

    var ratingPromptDismissed: Bool {
        didSet { defaults.set(ratingPromptDismissed, forKey: StorageKeys.ratingPromptDismissed) }
    }

    var ratingPromptNextCount: Int {
        didSet { defaults.set(ratingPromptNextCount, forKey: StorageKeys.ratingPromptNextCount) }
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

        codeFontFamily = FontFamily(rawValue: defaults.string(forKey: StorageKeys.codeFontFamily) ?? "") ?? .system
        let savedCodeFontSize = (defaults.number(forKey: StorageKeys.codeFontSize) ?? Self.defaultFontSize).clamped(to: Self.fontSizeRange)
        codeFontSize = savedCodeFontSize
        interfaceFontFamily = FontFamily(rawValue: defaults.string(forKey: StorageKeys.interfaceFontFamily) ?? "") ?? .system
        // Before the split one size drove interface text too, so seed it once to keep the interface from changing size.
        if let savedInterfaceFontSize = defaults.number(forKey: StorageKeys.interfaceFontSize) {
            interfaceFontSize = savedInterfaceFontSize.clamped(to: Self.fontSizeRange)
        } else {
            interfaceFontSize = savedCodeFontSize
            defaults.set(savedCodeFontSize, forKey: StorageKeys.interfaceFontSize)
        }
        codeLineSpacing = (defaults.number(forKey: StorageKeys.codeLineSpacing) ?? Self.lineSpacingRange.lowerBound)
            .clamped(to: Self.lineSpacingRange)
        codeLigatures = defaults.object(forKey: StorageKeys.codeLigatures) as? Bool ?? true
        appearanceMode = AppearanceMode(rawValue: defaults.string(forKey: StorageKeys.appearanceMode) ?? "") ?? .system
        launchLanguage = defaults.string(forKey: StorageKeys.language).flatMap { tag in
            availableLanguages().contains { $0.tag == tag } ? tag : nil
        }
        language = launchLanguage
        let lightPaletteId = defaults.string(forKey: StorageKeys.diffPaletteLight) ?? Self.defaultLightDiffPaletteId
        let darkPaletteId = defaults.string(forKey: StorageKeys.diffPaletteDark) ?? Self.defaultDarkDiffPaletteId
        diffPaletteLight = lightPaletteId
        diffPaletteDark = darkPaletteId
        lightDiffPalette = diffPalette(id: lightPaletteId, isDark: false)
        darkDiffPalette = diffPalette(id: darkPaletteId, isDark: true)
        sideBySideDiff = defaults.bool(forKey: StorageKeys.sideBySideDiff)
        ignoreWhitespace = defaults.bool(forKey: StorageKeys.ignoreWhitespace)
        hideGitLfsDiffs = defaults.object(forKey: StorageKeys.hideGitLfsDiffs) as? Bool ?? true
        enableGitSubmoduleSupport = defaults.object(forKey: StorageKeys.enableGitSubmoduleSupport) as? Bool ?? false
        hideReviewedFiles = defaults.bool(forKey: StorageKeys.hideReviewedFiles)
        treeFileList = defaults.bool(forKey: StorageKeys.treeFileList)
        autoExpandDescription = defaults.bool(forKey: StorageKeys.autoExpandDescription)
        tintWindowWithWallpaper = defaults.object(forKey: StorageKeys.tintWindowWithWallpaper) as? Bool ?? false
        skipAbandonConfirmation = defaults.bool(forKey: StorageKeys.skipAbandonConfirmation)
        skipWorkspaceDeleteConfirmation = defaults.bool(forKey: StorageKeys.skipWorkspaceDeleteConfirmation)
        confirmDragRebase = defaults.object(forKey: StorageKeys.confirmDragRebase) as? Bool ?? true
        sidebarWidth = min(max(defaults.number(forKey: StorageKeys.sidebarWidth) ?? 360, PaneLayout.sidebar.lowerBound), PaneLayout.sidebar.upperBound)
        let storedPaneWidth = defaults.number(forKey: StorageKeys.secondaryPaneWidth)
            ?? defaults.number(forKey: StorageKeys.legacyFileColumnWidth)
        secondaryPaneWidth = min(max(storedPaneWidth ?? PaneLayout.secondaryPaneDefault, PaneLayout.secondaryPane.lowerBound), PaneLayout.secondaryPane.upperBound)
        sidebarHidden = defaults.bool(forKey: StorageKeys.sidebarHidden)
        var seenRecentRepos = Set<String>()
        recentRepos = (defaults.stringArray(forKey: StorageKeys.recentRepos) ?? [])
            .filter { !$0.isEmpty }
            .map(Self.standardizedRepositoryPath)
            .filter { seenRecentRepos.insert($0).inserted }
        lastOpenedRepo = defaults.string(forKey: StorageKeys.lastOpenedRepo)
            .flatMap { $0.isEmpty ? nil : Self.standardizedRepositoryPath($0) }
        hasCompletedOnboarding = defaults.bool(forKey: StorageKeys.hasCompletedOnboarding)
        showsRecentRepositoriesPanel = defaults.bool(forKey: StorageKeys.showsRecentRepositoriesPanel)
        externalEditor = ExternalEditor(rawValue: defaults.string(forKey: StorageKeys.externalEditor) ?? "") ?? .vscode
        customEditorCommand = defaults.string(forKey: StorageKeys.customEditorCommand) ?? ""
        terminal = Terminal(rawValue: defaults.string(forKey: StorageKeys.terminal) ?? "") ?? .terminal
        customTerminalCommand = defaults.string(forKey: StorageKeys.customTerminalCommand) ?? ""
        aiProviderOrder = aiProviders(ids: defaults.stringArray(forKey: StorageKeys.aiProviderOrder) ?? [])
        ratingPromptActionCount = defaults.integer(forKey: StorageKeys.ratingPromptActionCount)
        ratingPromptDismissed = defaults.bool(forKey: StorageKeys.ratingPromptDismissed)
        ratingPromptNextCount = max(defaults.integer(forKey: StorageKeys.ratingPromptNextCount), 5)
        sendsAnonymousStats = defaults.object(forKey: Self.sendsAnonymousStatsKey) as? Bool ?? true
        updateChannel = parseUpdateChannel(value: defaults.string(forKey: Self.updateChannelKey) ?? "")
    }
}

private extension UserDefaults {
    /// Launch arguments store numbers as strings, which `as? Double` would drop.
    func number(forKey key: String) -> Double? {
        switch object(forKey: key) {
            case let number as NSNumber: number.doubleValue
            case let string as String: Double(string)
            default: nil
        }
    }
}

private extension Double {
    func clamped(to range: ClosedRange<Double>) -> Double {
        min(max(self, range.lowerBound), range.upperBound)
    }
}
