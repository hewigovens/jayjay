import XCTest

final class RepoListInitialRepositoryScene: SceneBase {
    func testRepositoryListReusesInitialWindow() throws {
        let app = try XCTUnwrap(app)
        XCTAssertEqual(app.windows.count, 1, "JayJay did not start with one repository window")

        let repoWindow = app.windows.firstMatch
        let titleMenu = repoWindow.toolbars.menuButtons["Switch Repository"].firstMatch
        XCTAssertTrue(titleMenu.waitForExistence(timeout: 5), "Repository title menu missing")
        titleMenu.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 1.2)).click()

        let repoList = app.menuItems["Repository List..."]
        XCTAssertTrue(repoList.waitForExistence(timeout: 3), "Repository List menu item missing")
        repoList.click()

        XCTAssertTrue(
            app.staticTexts["Recent Repositories"].waitForExistence(timeout: 5),
            "Repository title menu did not show the repository list"
        )
        XCTAssertEqual(app.windows.count, 1, "Opening the repository list duplicated the initial window")
    }
}
