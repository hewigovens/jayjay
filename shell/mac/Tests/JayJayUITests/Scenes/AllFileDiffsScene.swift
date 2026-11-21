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
        let stackedBelowRoot = app.buttons[AID.FileEditor.open("config/environments/development.json")]
        XCTAssertTrue(root.waitForExistence(timeout: 10))
        XCTAssertTrue(app.buttons[AID.FileEditor.open("a-root.txt")].waitForExistence(timeout: 10))
        XCTAssertFalse(root.isSelected, "A file was selected although every diff is shown")
        XCTAssertTrue(stackedBelowRoot.waitForExistence(timeout: 5), "The diffs were not stacked")

        root.click()
        XCTAssertTrue(root.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(stackedBelowRoot.waitForNonExistence(timeout: 5), "Selecting a file did not show it alone")

        keyStroke(.escape)
        XCTAssertTrue(root.wait(for: \.isSelected, toEqual: false, timeout: 5), "Escape did not return to every diff")
        XCTAssertTrue(stackedBelowRoot.waitForExistence(timeout: 5))

        root.click()
        app.buttons[AID.Diff.showAllFiles].click()
        XCTAssertTrue(root.wait(for: \.isSelected, toEqual: false, timeout: 5), "Show All Files did not return to every diff")
        XCTAssertTrue(stackedBelowRoot.waitForExistence(timeout: 5))

        keyStroke(.end)
        let showLargeDiff = app.buttons[AID.Diff.showLargeDiff]
        XCTAssertTrue(showLargeDiff.waitForExistence(timeout: 10), "A large diff was not folded in the stack")
        let editLong = app.buttons[AID.FileEditor.open("zy-long.txt")]
        XCTAssertTrue(editLong.waitForExistence(timeout: 5))
        XCTAssertTrue(
            app.windows.firstMatch.frame.contains(editLong.frame),
            "The header of the file in view scrolled away with its first lines"
        )
        showLargeDiff.click()
        XCTAssertTrue(generated.wait(for: \.isSelected, toEqual: true, timeout: 5), "Show Diff did not open the large diff alone")
        XCTAssertTrue(stackedBelowRoot.waitForNonExistence(timeout: 5))
    }

    func testStackedSideBySideDiffsScrollWithTheList() throws {
        let app = try XCTUnwrap(app)
        let editRoot = app.buttons[AID.FileEditor.open("a-root.txt")]
        XCTAssertTrue(editRoot.waitForExistence(timeout: 10))
        XCTAssertTrue(waitForHittable(editRoot, isHittable: true))
        app.buttons[AID.Diff.layoutToggle].click()

        let pane = app.textViews.matching(identifier: AID.Diff.sideBySide).firstMatch
        XCTAssertTrue(pane.waitForExistence(timeout: 10), "Stacked diffs did not switch to side-by-side")
        XCTAssertTrue(waitForHittable(pane, isHittable: true), "No side-by-side diff in view at the top of the stack")

        pane.scroll(byDeltaX: 0, deltaY: -2000)
        XCTAssertTrue(waitForHittable(editRoot, isHittable: false), "Scrolling over a side-by-side diff did not scroll the stack")
    }
}
