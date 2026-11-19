import XCTest

final class AllFileDiffsScene: SceneBase {
    override class var fixtureName: String {
        "all-file-diffs"
    }

    override class var additionalLaunchArguments: [String] {
        ["-jayjay.treeFileList", "NO", "-jayjay.showAllFileDiffs", "YES"]
    }

    func testEveryDiffShowsUntilAFileIsSelected() throws {
        let app = try XCTUnwrap(app)
        let root = fileRows(of: app).matching(identifier: AID.FileList.row("a-root.txt")).firstMatch
        let generated = fileRows(of: app).matching(identifier: AID.FileList.row("zz-generated.txt")).firstMatch
        let layout = app.buttons[AID.Diff.layoutToggle]
        XCTAssertTrue(root.waitForExistence(timeout: 10))
        XCTAssertTrue(app.buttons[AID.FileEditor.open("a-root.txt")].waitForExistence(timeout: 10))
        XCTAssertFalse(root.isSelected, "A file was selected although every diff is shown")
        XCTAssertFalse(layout.isEnabled, "The diffs were not stacked")

        root.click()
        XCTAssertTrue(root.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(layout.wait(for: \.isEnabled, toEqual: true, timeout: 5), "Selecting a file did not show it alone")

        keyStroke(.escape)
        XCTAssertTrue(root.wait(for: \.isSelected, toEqual: false, timeout: 5), "Escape did not return to every diff")
        XCTAssertTrue(layout.wait(for: \.isEnabled, toEqual: false, timeout: 5))

        keyStroke(.end)
        let showLargeDiff = app.buttons[AID.Diff.showLargeDiff]
        XCTAssertTrue(showLargeDiff.waitForExistence(timeout: 10), "A large diff was not folded in the stack")
        showLargeDiff.click()
        XCTAssertTrue(generated.wait(for: \.isSelected, toEqual: true, timeout: 5), "Show Diff did not open the large diff alone")
        XCTAssertTrue(layout.wait(for: \.isEnabled, toEqual: true, timeout: 5))
    }
}
