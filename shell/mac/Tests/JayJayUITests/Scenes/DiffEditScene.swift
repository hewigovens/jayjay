import XCTest

final class DiffEditScene: SceneBase {
    override class var fixtureName: String {
        "complex"
    }

    func testLargeRepositoryStartsCollapsedAndCanExpand() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let open = app.buttons[AID.DiffEdit.open]
        XCTAssertTrue(open.waitForExistence(timeout: 5), "Edit Diff button did not appear")
        open.click()

        let binaryCard = app.buttons[AID.DiffEdit.fileToggle("assets/logo.bin")]
        let configCard = app.buttons[AID.DiffEdit.fileToggle("config/environments/development.json")]
        XCTAssertTrue(binaryCard.waitForExistence(timeout: 5), "Binary diff-edit file card did not appear")
        XCTAssertTrue(configCard.waitForExistence(timeout: 5), "Text diff-edit file card did not appear")
        XCTAssertEqual(binaryCard.value as? String, "collapsed", "Large repository should start auto-collapsed")
        XCTAssertEqual(configCard.value as? String, "collapsed", "Large repository should start auto-collapsed")

        let expandAll = app.buttons[AID.DiffEdit.expandAll]
        XCTAssertTrue(expandAll.waitForExistence(timeout: 5), "Expand All button did not appear")
        expandAll.click()
        XCTAssertTrue(
            app.textViews[AID.Diff.text].firstMatch.waitForExistence(timeout: 10),
            "Expand All did not restore file previews"
        )

        let preview = app.textViews[AID.Diff.text].firstMatch
        let collapseAll = app.buttons[AID.DiffEdit.collapseAll]
        XCTAssertTrue(collapseAll.waitForExistence(timeout: 5), "Collapse All button did not appear")
        collapseAll.click()
        XCTAssertTrue(preview.waitForNonExistence(timeout: 5), "Collapse All did not hide file previews")

        let cancel = app.buttons[AID.DiffEdit.cancel]
        XCTAssertTrue(cancel.waitForExistence(timeout: 5), "Cancel button did not appear")
        cancel.click()
        XCTAssertTrue(open.waitForExistence(timeout: 5), "Cancel did not leave diff-edit mode")
    }
}
