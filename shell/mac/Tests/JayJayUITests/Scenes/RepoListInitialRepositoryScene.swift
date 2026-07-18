import XCTest

final class RepoListInitialRepositoryScene: SceneBase {
    override class var repositoryStoreFixtureName: String {
        "repositories-pinned.json"
    }

    func testPinnedRepositoryOpensInNewWindow() throws {
        let app = try XCTUnwrap(app)
        _ = try openPinnedRepository(in: app)
    }

    func testRepositoryListWorksAfterMainWindowCloses() throws {
        let app = try XCTUnwrap(app)
        let (mainWindow, pinnedWindow) = try openPinnedRepository(in: app)
        mainWindow.buttons[XCUIIdentifierCloseWindow].click()
        chooseRepositoryList(in: app, from: pinnedWindow)

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Secondary repository window did not open the repository list"
        )
    }

    func testRepositoryListReusesInitialWindow() throws {
        let app = try XCTUnwrap(app)
        XCTAssertEqual(app.windows.count, 1, "JayJay did not start with one repository window")

        let repoWindow = app.windows.firstMatch
        chooseRepositoryList(in: app, from: repoWindow)

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Repository title menu did not show the repository list"
        )
        XCTAssertEqual(app.windows.count, 1, "Opening the repository list duplicated the initial window")
    }

    private func openPinnedRepository(in app: XCUIApplication) throws -> (XCUIElement, XCUIElement) {
        let repoWindow = app.windows["simple"]
        XCTAssertTrue(repoWindow.waitForExistence(timeout: 5), "Initial repository window missing")
        openRepositoryTitleMenu(in: repoWindow)

        let pinnedRepos = app.menuItems.matching(identifier: "simple-formats")
        XCTAssertTrue(pinnedRepos.firstMatch.waitForExistence(timeout: 3), "Pinned repository menu item missing")
        let pinnedRepo = try XCTUnwrap(
            pinnedRepos.allElementsBoundByIndex.first(where: \.isHittable),
            "Pinned repository menu item was not actionable"
        )
        pinnedRepo.click()

        XCTAssertTrue(
            app.windows["simple-formats"].waitForExistence(timeout: 10),
            "Pinned repository did not open after the title menu closed"
        )
        return (repoWindow, app.windows["simple-formats"])
    }
}
