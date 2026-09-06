import XCTest

final class RepositoryPickerPinningScene: SceneBase {
    override class var repositoryStoreFixtureName: String {
        "repositories-picker-pinning.json"
    }

    func testPinCurrentRepositoryAndUnpinClosedRepository() throws {
        let app = try XCTUnwrap(app)
        let repoWindow = app.windows["simple"]
        let currentPath = try XCTUnwrap(fixtureURL).resolvingSymlinksInPath().path
        XCTAssertTrue(repoWindow.waitForExistence(timeout: 5))

        for action in ["Pin", "Unpin"] {
            openRepositoryTitlePicker(in: repoWindow)
            let row = app.descendants(matching: .any)[AID.Picker.row("repo-\(currentPath)")].firstMatch
            XCTAssertTrue(row.waitForExistence(timeout: 5))
            row.rightClick()
            let item = app.menuItems[action]
            XCTAssertTrue(item.waitForExistence(timeout: 5))
            let screenshot = XCTAttachment(screenshot: repoWindow.screenshot())
            screenshot.name = "Repository picker - \(action)"
            screenshot.lifetime = .keepAlways
            add(screenshot)
            item.click()
            XCTAssertTrue(row.waitForNonExistence(timeout: 5))
            XCTAssertEqual(app.windows.count, 1)
        }

        openRepositoryTitlePicker(in: repoWindow)
        let closedRow = app.buttons.matching(NSPredicate(
            format: "identifier BEGINSWITH %@ AND identifier ENDSWITH %@",
            AID.Picker.row("repo-"),
            "/formats"
        )).firstMatch
        XCTAssertTrue(closedRow.waitForExistence(timeout: 5))
        closedRow.rightClick()
        let unpin = app.menuItems["Unpin"]
        XCTAssertTrue(unpin.waitForExistence(timeout: 5))
        unpin.click()
        XCTAssertTrue(closedRow.waitForNonExistence(timeout: 5))
        openRepositoryTitlePicker(in: repoWindow)
        XCTAssertFalse(closedRow.exists)
        keyStroke(.escape)
        XCTAssertEqual(app.windows.count, 1)
    }
}
