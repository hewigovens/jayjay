import SwiftUI

struct OnboardingScene: Scene {
    let launchScene: LaunchScene
    let settings: AppSettings
    let windowManager: RepoWindowManager

    var body: some Scene {
        Window("Welcome to JayJay", id: AppWindows.onboarding) {
            OnboardingView { windowManager.finishOnboarding() }
                .appEnvironment(settings)
                .background(RepoListWindowBridge(windowManager: windowManager, scene: AppWindows.onboarding))
                .background(WindowConfigurator { window in
                    window.identifier = NSUserInterfaceItemIdentifier(AppWindows.onboarding)
                })
        }
        .handlesExternalEvents(matching: [])
        .windowResizability(.contentSize)
        .defaultSize(OnboardingView.preferredSize)
        .defaultLaunchBehavior(launchScene.onboardingBehavior)
        .restorationBehavior(.disabled)
    }
}
