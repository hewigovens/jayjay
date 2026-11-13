import XCTest

final class FolderReviewScene: SceneBase {
    override class var fixtureName: String {
        "file-selection-scroll"
    }

    override class var additionalLaunchArguments: [String] {
        ["-jayjay.treeFileList", "YES"]
    }

    func testFolderMenuMarksAndUnmarksItsFiles() throws {
        let app = try XCTUnwrap(app)
        let folder = app.buttons["config/environments"]
        let development = app.buttons[AID.FileList.review("config/environments/development.json")]
        let staging = app.buttons[AID.FileList.review("config/environments/staging.json")]
        XCTAssertTrue(development.waitForExistence(timeout: 10))
        development.click()
        XCTAssertTrue(development.wait(for: \.label, toEqual: "Reviewed", timeout: 5))

        folder.rightClick()
        app.menuItems["Mark 2 Files as Reviewed"].click()
        XCTAssertTrue(staging.wait(for: \.label, toEqual: "Reviewed", timeout: 5))
        XCTAssertEqual(development.label, "Reviewed")

        folder.rightClick()
        app.menuItems["Mark 2 Files as Unreviewed"].click()
        XCTAssertTrue(staging.wait(for: \.label, toEqual: "Unreviewed", timeout: 5))
        XCTAssertEqual(development.label, "Unreviewed")
    }
}
