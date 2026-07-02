import XCTest

final class CommitBoxSaveScene: SceneBase {
    /// Dedicated fixture: saving @'s description rewrites the working-copy commit.
    override class var fixtureName: String {
        "simple-save-description"
    }

    func testSaveDescriptionDoesNotAdvanceWorkingCopy() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let originalTopId = rows.element(boundBy: 0).identifier

        let summary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(summary.waitForExistence(timeout: 5), "CommitBox summary field not found")
        let typed = "saved working copy description"
        summary.click()
        summary.typeText(typed)
        // Synthesized typing can race the app's cold-start re-renders and drop keystrokes; re-type once if needed.
        if (summary.value as? String) != typed {
            summary.click()
            app.typeKey("a", modifierFlags: .command)
            summary.typeText(typed)
        }
        XCTAssertEqual(summary.value as? String, typed, "Summary text did not land in the field")

        let save = app.buttons[AID.CommitBox.save]
        XCTAssertTrue(save.waitForExistence(timeout: 3), "Describe button not found")
        save.click()

        XCTAssertTrue(
            app.staticTexts[typed].waitForExistence(timeout: 10),
            "Saved description did not appear in the detail view"
        )
        XCTAssertEqual(
            rows.element(boundBy: 0).identifier,
            originalTopId,
            "Save should describe @ without creating a new working-copy change"
        )
    }
}
