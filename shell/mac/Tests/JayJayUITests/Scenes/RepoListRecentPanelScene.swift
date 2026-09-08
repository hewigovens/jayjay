import XCTest

final class RepoListRecentPanelScene: SceneBase {
    override class var opensFixtureOnLaunch: Bool {
        false
    }

    override class var repositoryStoreFixtureName: String {
        "repositories-recent-panel.json"
    }

    func testRecentPanelTogglesAndRemembersItsState() throws {
        let app = try XCTUnwrap(app)
        let recents = app.staticTexts["Recent Repositories"]
        let pinned = app.buttons.matching(NSPredicate(format: "label BEGINSWITH %@", "simple,")).firstMatch
        XCTAssertTrue(recents.waitForExistence(timeout: 5), "Recent panel did not show on a fresh launch")
        XCTAssertTrue(pinned.exists, "Pinned repository is missing beside the hero")

        let window = app.windows["JayJay"]
        window.toolbars.buttons["Hide Sidebar"].click()
        XCTAssertTrue(recents.waitForNonExistence(timeout: 5), "Hide Sidebar did not hide the Recent panel")
        XCTAssertTrue(pinned.exists, "Hiding the Recent panel removed the pinned repository")

        app.terminate()
        app.launch()
        XCTAssertTrue(pinned.waitForExistence(timeout: 10), "Repository list did not come back after relaunch")
        XCTAssertFalse(recents.exists, "Recent panel forgot that it was hidden")

        window.toolbars.buttons["Show Sidebar"].click()
        XCTAssertTrue(recents.waitForExistence(timeout: 5), "Show Sidebar did not bring the Recent panel back")
    }

    func testUnpinningReturnsARepositoryToRecent() throws {
        let app = try XCTUnwrap(app)
        let pinned = app.buttons.matching(NSPredicate(format: "label BEGINSWITH %@", "simple,")).firstMatch
        XCTAssertTrue(pinned.waitForExistence(timeout: 5), "Pinned repository did not appear")
        XCTAssertTrue(app.staticTexts["Pinned"].exists)

        app.buttons["Remove Pin"].firstMatch.click()

        XCTAssertTrue(app.staticTexts["Pinned"].waitForNonExistence(timeout: 5), "Unpinning left the Pinned section")
        XCTAssertTrue(pinned.waitForExistence(timeout: 5), "Unpinned repository vanished instead of joining Recent")
    }
}
