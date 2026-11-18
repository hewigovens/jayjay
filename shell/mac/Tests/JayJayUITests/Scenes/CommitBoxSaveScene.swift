import XCTest

final class CommitBoxSaveScene: SceneBase {
    override class var fixtureName: String {
        "save-description"
    }

    func testDescribeThenCommitFromWorkingCopySheet() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let originalTopId = rows.element(boundBy: 0).identifier

        let summary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(summary.waitForExistence(timeout: 5), "CommitBox summary field not found")
        let typed = "saved working copy description"
        summary.click()
        paste(typed)
        // Synthesized input can race the app's cold-start re-renders; retry once if needed.
        if (summary.value as? String) != typed {
            summary.click()
            app.typeKey("a", modifierFlags: .command)
            paste(typed)
        }
        XCTAssertEqual(summary.value as? String, typed, "Summary text did not land in the field")

        let save = app.buttons[AID.CommitBox.save]
        XCTAssertTrue(save.waitForExistence(timeout: 3), "Describe button not found")
        save.click()

        let description = app.staticTexts
            .matching(identifier: AID.Detail.descriptionTitle).matching(NSPredicate(format: "value == %@", typed)).firstMatch
        XCTAssertTrue(
            description.waitForExistence(timeout: 10),
            "Saved description did not appear in the detail view"
        )
        XCTAssertTrue(summary.waitForNonExistence(timeout: 5), "The saved working copy still shows the sidebar commit box")
        app.buttons[AID.Detail.editDescription].click()
        let sheet = app.sheets.firstMatch
        let editor = sheet.textFields[AID.CommitBox.summary]
        XCTAssertTrue(editor.waitForExistence(timeout: 5), "Edit did not open the working-copy description sheet")
        XCTAssertEqual(editor.value as? String, typed, "The sheet did not load the saved description")
        editor.click()
        app.typeKey("a", modifierFlags: .command)
        let revised = "revised working copy description"
        paste(revised)
        sheet.buttons["Save"].click()

        let updated = app.staticTexts.matching(identifier: AID.Detail.descriptionTitle)
            .matching(NSPredicate(format: "value == %@", revised)).firstMatch
        XCTAssertTrue(updated.waitForExistence(timeout: 10), "Save did not update the description in the header")
        XCTAssertFalse(summary.exists, "Saving the edited description reopened the commit box")
        XCTAssertEqual(
            rows.element(boundBy: 0).identifier,
            originalTopId,
            "Save should describe @ without creating a new working-copy change"
        )

        app.buttons[AID.Detail.editDescription].click()
        editor.click()
        app.typeKey("a", modifierFlags: .command)
        let finalMessage = "committed from the description sheet"
        paste(finalMessage)
        sheet.buttons[AID.CommitBox.commit].click()
        XCTAssertTrue(sheet.waitForNonExistence(timeout: 10), "Commit left the description sheet open")
        let committed = rows.matching(identifier: originalTopId).matching(NSPredicate(format: "value CONTAINS %@", finalMessage)).firstMatch
        XCTAssertTrue(committed.waitForExistence(timeout: 10), "Commit did not use the edited sheet message")
        XCTAssertTrue(summary.waitForExistence(timeout: 10), "The commit box did not return for the new working copy")
        XCTAssertEqual(summary.value as? String, "", "The new working copy inherited the committed summary")
        XCTAssertNotEqual(rows.element(boundBy: 0).identifier, originalTopId, "Commit did not advance the working copy")
    }

    func testContextMenuCommitsADescribedWorkingCopy() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")
        let originalTopId = rows.element(boundBy: 0).identifier
        let summary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(summary.waitForExistence(timeout: 5), "CommitBox summary field not found")
        summary.click()
        paste("described before committing")
        app.buttons[AID.CommitBox.save].click()
        XCTAssertTrue(summary.waitForNonExistence(timeout: 10), "The saved working copy still shows the sidebar commit box")

        rows.element(boundBy: 0).rightClick()
        let commit = app.windows.menuItems["Commit"]
        XCTAssertTrue(commit.waitForExistence(timeout: 3), "A described working copy did not offer Commit in its menu")
        commit.click()

        XCTAssertTrue(summary.waitForExistence(timeout: 10), "The commit box did not return for the new working copy")
        XCTAssertNotEqual(rows.element(boundBy: 0).identifier, originalTopId, "Commit did not advance the working copy")
    }
}
