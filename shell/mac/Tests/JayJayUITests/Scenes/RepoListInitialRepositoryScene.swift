import XCTest

final class RepoListInitialRepositoryScene: SceneBase {
    override class var repositoryStoreFixtureName: String {
        "repositories-pinned.json"
    }

    func testPinnedRepositoryOpensInNewWindow() throws {
        let app = try XCTUnwrap(app)
        _ = openPinnedRepository(in: app)
    }

    func testClosingSoleInitialRepositoryWindowShowsRepoList() throws {
        let app = try XCTUnwrap(app)
        XCTAssertEqual(app.windows.count, 1, "JayJay did not start with one repository window")

        let initialWindow = app.windows["simple"]
        XCTAssertTrue(initialWindow.waitForExistence(timeout: 5), "Initial repository window missing")
        initialWindow.buttons[XCUIIdentifierCloseWindow].click()

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Closing the sole initial repository window did not show the repository list"
        )
        XCTAssertTrue(
            app.windows["simple"].waitForNonExistence(timeout: 5),
            "The closed repository window was reactivated as the list instead of a fresh list window replacing it"
        )

        let simpleRow = app.buttons
            .matching(NSPredicate(format: "label BEGINSWITH %@", "simple,"))
            .firstMatch
        XCTAssertTrue(simpleRow.waitForExistence(timeout: 5), "Recent repository row missing from the list")
        simpleRow.click()

        let reopened = app.windows["simple"]
        XCTAssertTrue(reopened.waitForExistence(timeout: 10), "Reopening from the list did not open a repository window")
        XCTAssertTrue(
            repositoryTitleButton(in: reopened).waitForExistence(timeout: 10),
            "Reopened window is not a functional repository window"
        )
    }

    func testRepositoryListWorksAfterMainWindowCloses() throws {
        let app = try XCTUnwrap(app)
        let (mainWindow, pinnedWindow) = openPinnedRepository(in: app)

        app.menuBars.menuBarItems["Window"].click()
        let mainWindowMenuItems = app.menuItems.matching(identifier: "simple")
        XCTAssertTrue(mainWindowMenuItems.firstMatch.waitForExistence(timeout: 3), "Initial repository window menu item missing")
        let mainWindowMenuItem = try XCTUnwrap(
            mainWindowMenuItems.allElementsBoundByIndex.first(where: \.isHittable),
            "Initial repository window menu item was not actionable"
        )
        mainWindowMenuItem.click()
        app.typeKey("w", modifierFlags: .command)

        XCTAssertFalse(mainWindow.waitForExistence(timeout: 3), "Initial repository window did not close")
        XCTAssertTrue(pinnedWindow.waitForExistence(timeout: 3), "Secondary repository window closed unexpectedly")
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

    private func openPinnedRepository(in app: XCUIApplication) -> (XCUIElement, XCUIElement) {
        let repoWindow = app.windows["simple"]
        XCTAssertTrue(repoWindow.waitForExistence(timeout: 5), "Initial repository window missing")
        openRepositoryTitlePicker(in: repoWindow)

        // The row id embeds the fixture's absolute path.
        let pinnedRepo = app.buttons.matching(
            NSPredicate(
                format: "identifier BEGINSWITH %@ AND identifier ENDSWITH %@",
                AID.Picker.row("repo-"),
                "/formats"
            )
        ).firstMatch
        XCTAssertTrue(pinnedRepo.waitForExistence(timeout: 3), "Pinned repository row missing")
        pinnedRepo.click()

        XCTAssertTrue(
            app.windows["formats"].waitForExistence(timeout: 10),
            "Pinned repository did not open after the picker closed"
        )
        return (repoWindow, app.windows["formats"])
    }
}
