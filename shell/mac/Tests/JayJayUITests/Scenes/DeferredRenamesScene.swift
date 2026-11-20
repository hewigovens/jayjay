import XCTest

final class DeferredRenamesScene: SceneBase {
    override class var fixtureName: String {
        "many-renames"
    }

    override class var additionalLaunchArguments: [String] {
        ["-jayjay.treeFileList", "NO"]
    }

    func testRenamesReplaceTheFilesListedFirst() throws {
        let app = try XCTUnwrap(app)
        let renamed = fileRows(of: app).matching(identifier: AID.FileList.row("new/0.txt")).firstMatch
        XCTAssertTrue(renamed.waitForExistence(timeout: 10))
        let removed = fileRows(of: app).matching(identifier: AID.FileList.row("old/0.txt")).firstMatch
        XCTAssertTrue(removed.waitForNonExistence(timeout: 10), "The removed half of a rename stayed listed")
        XCTAssertTrue(app.staticTexts["Renamed"].firstMatch.waitForExistence(timeout: 10), "The file was not shown as a rename")
    }
}
