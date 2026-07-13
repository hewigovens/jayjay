import XCTest

final class RepoListWindowScene: SceneBase {
    override class var opensFixtureOnLaunch: Bool {
        false
    }

    func testOpeningRepositoryKeepsToolbar() throws {
        let app = try XCTUnwrap(app)
        let repoWindow = openSeededRepo(in: app)
        let initialFrame = repoWindow.frame
        XCTAssertGreaterThanOrEqual(initialFrame.width, 1000, "Repository window did not use its default width")
        XCTAssertGreaterThanOrEqual(initialFrame.height, 650, "Repository window did not use its default height")
        XCTAssertTrue(
            repoWindow.toolbars.buttons["Filter"].waitForExistence(timeout: 10),
            "Repository window lost its SwiftUI toolbar"
        )
        XCTAssertEqual(repoWindow.frame, initialFrame, "Repository window resized while installing its toolbar")
        XCTAssertFalse(app.staticTexts["Recent Repositories"].exists, "Repository list remained visible")
    }

    func testClosingLastRepositoryReturnsToRepoList() throws {
        let app = try XCTUnwrap(app)
        let repoWindow = openSeededRepo(in: app)
        repoWindow.buttons[XCUIIdentifierCloseWindow].click()

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Repository list did not return after closing the last repository"
        )
    }

    func testDockClickDoesNotDuplicateRepoListWindow() throws {
        let app = try XCTUnwrap(app)
        XCTAssertEqual(app.windows.count, 1, "JayJay did not start with one repository list")

        let dock = XCUIApplication(bundleIdentifier: "com.apple.dock")
        let dockItem = dock.dockItems.matching(identifier: "JayJay").firstMatch
        XCTAssertTrue(dockItem.waitForExistence(timeout: 3), "JayJay was not present in the Dock")
        dockItem.click()

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Repository list disappeared when the Dock item was clicked"
        )
        XCTAssertEqual(app.windows.count, 1, "Clicking the Dock item opened duplicate repository lists")
    }

    private func openSeededRepo(in app: XCUIApplication) -> XCUIElement {
        let recentRepo = app.staticTexts["simple-formats"]
        XCTAssertTrue(recentRepo.waitForExistence(timeout: 5), "Seeded repository did not appear")
        recentRepo.click()

        let repoWindow = app.windows["simple-formats"]
        XCTAssertTrue(repoWindow.waitForExistence(timeout: 10), "Repository window did not appear")
        return repoWindow
    }
}
