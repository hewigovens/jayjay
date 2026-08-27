import XCTest

final class OnboardingScene: OnboardingSceneBase {
    override class var opensFixtureOnLaunch: Bool {
        false
    }

    override class var additionalLaunchArguments: [String] {
        super.additionalLaunchArguments + ["-jayjay.lastOpenedRepo", ""]
    }

    func testOnboardingLeadsToRepositoryList() throws {
        let app = try XCTUnwrap(app)
        let onboarding = onboardingWindow(in: app)
        XCTAssertTrue(onboarding.waitForExistence(timeout: 10), "Onboarding window did not appear")
        XCTAssertEqual(app.windows.count, 1, "Another window opened beside onboarding")

        finishOnboarding(in: app)

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 10),
            "Finishing onboarding did not show the repository list"
        )
        XCTAssertTrue(onboarding.waitForNonExistence(timeout: 5), "Onboarding window stayed open")
        XCTAssertEqual(app.windows.count, 1, "Finishing onboarding left more than the repository list")
    }

    func testDockClickReopensOnboarding() throws {
        let app = try XCTUnwrap(app)
        let onboarding = onboardingWindow(in: app)
        XCTAssertTrue(onboarding.waitForExistence(timeout: 10), "Onboarding window did not appear")
        onboarding.buttons[XCUIIdentifierCloseWindow].click()
        XCTAssertTrue(waitForWindowCount(0, in: app), "Onboarding window did not close")

        clickDockItem()

        XCTAssertTrue(onboarding.waitForExistence(timeout: 5), "Dock click did not bring onboarding back")
        XCTAssertFalse(app.staticTexts["Recent Repositories"].exists, "Dock click bypassed onboarding")
        XCTAssertEqual(app.windows.count, 1, "Dock click opened more than the onboarding window")
    }
}
