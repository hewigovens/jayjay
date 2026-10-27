import JayJayCore
import SwiftUI

@main
struct JayJayApp: App {
    @NSApplicationDelegateAdaptor private var appDelegate: AppDelegate
    @State private var settings = AppSettings()
    @State private var repositoryStore = RepositoryStore()
    @State private var windowManager: RepoWindowManager
    private let launchScene: LaunchScene
    private let updater: SparkleUpdater

    init() {
        CommandLineInterface.runAndExitIfNeeded(arguments: CommandLine.arguments)
        // Writing to an exited child (ssh, jj) must fail with EPIPE instead of killing the app.
        signal(SIGPIPE, SIG_IGN)
        installLogFile(version: AppMetadata.compactVersionLabel)
        setPreferredLanguages(languages: Locale.preferredLanguages)
        setNumberLocale(locale: Locale.current.identifier(.bcp47))
        // With saved state consulted, SwiftUI presents the previous session's scenes instead of the launch route.
        UserDefaults.standard.register(defaults: ["ApplePersistenceIgnoreState": true])

        NSWindow.allowsAutomaticWindowTabbing = false
        if let helpers = Bundle.main.resourceURL?.appendingPathComponent("ssh-helpers") {
            setSshHelperDirectory(path: helpers.path)
        }

        let tool = ExternalToolInvocation.parse(arguments: CommandLine.arguments)
        let initialSettings = AppSettings()
        setDebugLogging(enabled: DebugMode.isEnabled)
        updater = SparkleUpdater(includesBetaUpdates: { initialSettings.updateChannel == .beta })
        if tool == nil, initialSettings.hasCompletedOnboarding {
            AppTelemetry.maybePing(enabled: initialSettings.sendsAnonymousStats)
        }
        let cliPath = LaunchArguments.repoPath(from: CommandLine.arguments)
        _settings = State(initialValue: initialSettings)
        let manager = RepoWindowManager(settings: initialSettings)
        _windowManager = State(initialValue: manager)
        launchScene = LaunchScene(
            isExternalTool: tool != nil,
            hasCompletedOnboarding: initialSettings.hasCompletedOnboarding,
            initialPath: tool == nil ? (cliPath ?? initialSettings.lastOpenedRepo) : nil
        )
        manager.pendingRepoAfterOnboarding = launchScene.onboardingNextRepo
        manager.launchScene = launchScene
        appDelegate.settings = initialSettings
        appDelegate.externalToolInvocation = tool
        appDelegate.openRepositoryPicker = { manager.openRepositoryPicker() }
        appDelegate.openHandler = { manager.openRepo($0) }
        appDelegate.showRepoSelector = { manager.showRepoList() }
        appDelegate.reopenWindows = { manager.reopen() }
        appDelegate.recentReposProvider = { initialSettings.recentRepos }
        appDelegate.prepareForTermination = { manager.prepareForTermination() }
    }

    var body: some Scene {
        OnboardingScene(launchScene: launchScene, settings: settings, windowManager: windowManager)
        RepoListScene(
            launchScene: launchScene,
            settings: settings,
            repositoryStore: repositoryStore,
            windowManager: windowManager
        )
        RepoWindowGroup(
            launchScene: launchScene,
            settings: settings,
            repositoryStore: repositoryStore,
            windowManager: windowManager,
            updater: updater
        )
        OverviewWindowScene(settings: settings, windowManager: windowManager)
        OpenRemoteRepositoryWindowScene(settings: settings, windowManager: windowManager)
        CloneRepositoryWindowScene(settings: settings, windowManager: windowManager)
        FileHistoryWindowScene(settings: settings, windowManager: windowManager)
        AppInfoScenes(settings: settings, updater: updater, windowManager: windowManager)
    }
}
