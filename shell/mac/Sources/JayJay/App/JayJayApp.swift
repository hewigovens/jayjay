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
        // With saved state consulted, SwiftUI presents the previous session's scenes instead of the launch route.
        UserDefaults.standard.register(defaults: ["ApplePersistenceIgnoreState": true])

        NSWindow.allowsAutomaticWindowTabbing = false

        let tool = ExternalToolInvocation.parse(arguments: CommandLine.arguments)
        let initialSettings = AppSettings()
        updater = SparkleUpdater(includesBetaUpdates: { initialSettings.updateChannel == .beta })
        if tool == nil {
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
        appDelegate.externalToolInvocation = tool
        appDelegate.openRepositoryPicker = { manager.openRepositoryPicker() }
        appDelegate.openHandler = { manager.openRepo($0) }
        appDelegate.showRepoSelector = { manager.showRepoList() }
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
        AppInfoScenes(settings: settings, updater: updater)
    }
}
