import XCTest

final class OnboardingLaunchRepositoryScene: OnboardingSceneBase {
    func testOnboardingOpensLaunchRepository() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(onboardingWindow(in: app).waitForExistence(timeout: 10), "Onboarding window did not appear")
        XCTAssertEqual(app.windows.count, 1, "The launch repository opened behind onboarding")

        finishOnboarding(in: app)

        XCTAssertTrue(
            app.windows["simple"].waitForExistence(timeout: 10),
            "Finishing onboarding dropped the launch repository"
        )
        XCTAssertFalse(
            app.staticTexts["Recent Repositories"].exists,
            "The repository list opened instead of the launch repository"
        )
        XCTAssertEqual(app.windows.count, 1, "Finishing onboarding left more than the launch repository")
    }
}
