import XCTest

final class RepoListWindowScene: SceneBase {
    override class var opensFixtureOnLaunch: Bool {
        false
    }

    override class var repositoryStoreFixtureName: String {
        "repositories-simple.json"
    }

    func testOpeningRepositoryKeepsToolbar() throws {
        let app = try XCTUnwrap(app)
        let repoWindow = openRepo(named: "formats", in: app)
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
        let repoWindow = openRepo(named: "formats", in: app)
        repoWindow.buttons[XCUIIdentifierCloseWindow].click()

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Repository list did not return after closing the last repository"
        )
    }

    func testClosingAllRepositoriesShowsRepoListOnce() throws {
        let app = try XCTUnwrap(app)
        let pinned = openRepo(named: "simple", in: app)
        chooseRepositoryList(in: app, from: pinned)
        let recent = openRepo(named: "formats", in: app)
        XCTAssertTrue(waitForWindowCount(2, in: app), "Opening a second repository did not leave both windows open")

        // Overlapping windows on a small screen stack their close buttons; close through the Window menu instead.
        try activateWindow(named: "simple", in: app)
        app.typeKey("w", modifierFlags: .command)
        XCTAssertTrue(pinned.waitForNonExistence(timeout: 5), "The first repository window did not close")
        XCTAssertFalse(
            app.staticTexts["Recent Repositories"].exists,
            "Repository list returned while a repository was still open"
        )

        try activateWindow(named: "formats", in: app)
        app.typeKey("w", modifierFlags: .command)
        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Repository list did not return after closing every repository"
        )
        XCTAssertEqual(app.windows.count, 1, "Closing every repository opened more than one repository list")
    }

    func testClosingRepoListKeepsRepositoryWindow() throws {
        let app = try XCTUnwrap(app)
        let repoWindow = openRepo(named: "formats", in: app)
        chooseRepositoryList(in: app, from: repoWindow)

        let repoList = app.windows["JayJay"]
        XCTAssertTrue(repoList.waitForExistence(timeout: 5), "Repository title menu did not open the repository list")
        repoList.buttons[XCUIIdentifierCloseWindow].click()

        XCTAssertTrue(repoList.waitForNonExistence(timeout: 5), "The repository list did not close")
        XCTAssertTrue(repoWindow.exists, "Closing the repository list closed the repository window")
        XCTAssertEqual(app.state, .runningForeground, "Closing the repository list quit JayJay")
    }

    func testRepositoryTitleMenuReturnsToRepoList() throws {
        let app = try XCTUnwrap(app)
        let repoWindow = openRepo(named: "formats", in: app)
        chooseRepositoryList(in: app, from: repoWindow)

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Repository title menu did not return to the repository list"
        )
    }

    func testDockClickDoesNotDuplicateRepoListWindow() throws {
        let app = try XCTUnwrap(app)
        XCTAssertEqual(app.windows.count, 1, "JayJay did not start with one repository list")

        clickDockItem()

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Repository list disappeared when the Dock item was clicked"
        )
        XCTAssertEqual(app.windows.count, 1, "Clicking the Dock item opened duplicate repository lists")
    }

    func testFileMenuIsAvailableWithoutRepositoryWindow() throws {
        let app = try XCTUnwrap(app)
        XCTAssertEqual(app.windows.count, 1, "JayJay did not start with one repository list")

        app.menuBars.menuBarItems["File"].click()

        XCTAssertTrue(app.menuItems["Open Repository..."].waitForExistence(timeout: 3), "File menu lost Open Repository")
        XCTAssertTrue(app.menuItems["Open Recent"].exists, "File menu lost Open Recent")
        keyStroke(.escape)
    }

    func testRepositoryReopenedAfterClosingTheLastOneLoadsItsDetail() throws {
        let app = try XCTUnwrap(app)
        let first = openRepo(named: "formats", in: app)
        XCTAssertTrue(dagRows(of: app).firstMatch.waitForExistence(timeout: 10), "First repository never loaded")
        try activateWindow(named: "formats", in: app)
        app.typeKey("w", modifierFlags: .command)
        XCTAssertTrue(first.waitForNonExistence(timeout: 5), "First repository window did not close")
        XCTAssertTrue(app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5), "Repository list did not return")

        let reopened = openRepo(named: "simple", in: app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10), "Reopened repository never loaded its DAG")
        XCTAssertTrue(
            fileRows(of: app).firstMatch.waitForExistence(timeout: 10),
            "Reopened repository did not load the selected change's files"
        )
        XCTAssertFalse(app.staticTexts["Select a Change"].exists, "Reopened repository shows no selected change")
        XCTAssertLessThan(
            rows.firstMatch.frame.minY - reopened.frame.minY,
            200,
            "DAG rows are pushed down the sidebar: \(rows.firstMatch.frame) in \(reopened.frame)"
        )
    }

    func testDockClickWithNoWindowsShowsRepoList() throws {
        let app = try XCTUnwrap(app)
        app.windows.firstMatch.buttons[XCUIIdentifierCloseWindow].click()
        XCTAssertTrue(waitForWindowCount(0, in: app), "The repository list window did not close")

        clickDockItem()

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Clicking the Dock item with no windows did not show the repository list"
        )
        XCTAssertEqual(app.windows.count, 1, "Clicking the Dock item opened more than one repository list")
    }

    func testDockClickDeminiaturizesRepoList() throws {
        let app = try XCTUnwrap(app)
        let repoList = app.windows.firstMatch
        let recents = app.staticTexts["Recent Repositories"]
        XCTAssertTrue(recents.waitForExistence(timeout: 5), "Repository list did not appear")
        repoList.buttons[XCUIIdentifierMinimizeWindow].click()
        XCTAssertTrue(waitForHittable(recents, isHittable: false), "The repository list did not miniaturize")

        clickDockItem()

        XCTAssertTrue(repoList.waitForExistence(timeout: 5), "The miniaturized repository list did not come back")
        XCTAssertTrue(waitForHittable(recents, isHittable: true), "The repository list stayed miniaturized in the Dock")
        XCTAssertEqual(app.windows.count, 1, "Deminiaturizing opened a second repository list")
    }

    private func openRepo(named name: String, in app: XCUIApplication) -> XCUIElement {
        let row = app.buttons
            .matching(NSPredicate(format: "label BEGINSWITH %@", "\(name),"))
            .firstMatch
        XCTAssertTrue(row.waitForExistence(timeout: 5), "Seeded repository \(name) did not appear")
        row.click()

        let repoWindow = app.windows[name]
        XCTAssertTrue(repoWindow.waitForExistence(timeout: 10), "Repository window \(name) did not appear")
        return repoWindow
    }
}
