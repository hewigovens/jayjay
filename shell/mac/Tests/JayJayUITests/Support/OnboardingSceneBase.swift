import XCTest

/// The fixture defaults mark onboarding complete, so these scenes re-arm it through the argument domain.
class OnboardingSceneBase: SceneBase {
    override class var additionalLaunchArguments: [String] {
        ["-jayjay.hasCompletedOnboarding", "NO"]
    }

    func onboardingWindow(in app: XCUIApplication) -> XCUIElement {
        app.windows["Welcome to JayJay"]
    }

    func finishOnboarding(in app: XCUIApplication) {
        let next = app.buttons["Next"]
        XCTAssertTrue(next.waitForExistence(timeout: 10), "Onboarding did not offer its first page")
        for _ in 0 ..< 5 where next.exists {
            next.click()
        }
        clickCenter(app.buttons["Get Started"], message: "Onboarding did not reach Get Started")
    }
}
