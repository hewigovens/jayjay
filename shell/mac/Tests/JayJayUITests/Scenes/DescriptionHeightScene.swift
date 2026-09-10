import XCTest

final class DescriptionHeightScene: SceneBase {
    override class var fixtureName: String {
        "description-height"
    }

    func testDescriptionFitsContentAndExpandsAcrossChanges() throws {
        let app = try XCTUnwrap(app)
        let description = app.scrollViews[AID.Detail.description]
        let toggle = app.buttons[AID.Detail.descriptionExpansion]

        selectChange("short description", in: app)
        let shortHeight = description.frame.height
        XCTAssertFalse(toggle.exists)

        selectChange("multiline description", in: app)
        let multilineHeight = description.frame.height
        XCTAssertGreaterThan(multilineHeight, shortHeight + 20)
        XCTAssertFalse(toggle.exists)
        XCTAssertLessThanOrEqual(description.staticTexts.firstMatch.frame.height, multilineHeight + 2)

        selectChange("long description", in: app)
        let collapsedHeight = description.frame.height
        XCTAssertGreaterThan(collapsedHeight, multilineHeight)
        XCTAssertGreaterThan(description.staticTexts.firstMatch.frame.height, collapsedHeight)
        XCTAssertTrue(toggle.waitForExistence(timeout: 5))
        toggle.click()
        XCTAssertGreaterThan(description.frame.height, collapsedHeight + 40)
        XCTAssertEqual(toggle.label, "Collapse description")
        XCTAssertTrue(toggle.isHittable)
        toggle.click()
        XCTAssertEqual(description.frame.height, collapsedHeight, accuracy: 2)

        selectChange("wrapped description", in: app)
        XCTAssertEqual(description.frame.height, collapsedHeight, accuracy: 2)
        toggle.click()
        XCTAssertGreaterThan(description.frame.height, collapsedHeight + 40)
        XCTAssertTrue(app.windows.firstMatch.frame.contains(description.frame))
        XCTAssertTrue(toggle.isHittable)

        for (message, expected) in [
            ("short description", shortHeight),
            ("multiline description", multilineHeight),
            ("long description", collapsedHeight)
        ] {
            selectChange(message, in: app)
            XCTAssertEqual(description.frame.height, expected, accuracy: 2)
        }
    }

    func testDescriptionEditorFitsDraftAndSaves() throws {
        let app = try XCTUnwrap(app)
        selectChange("editable description", in: app)
        app.buttons["Edit"].click()
        let editor = app.textViews[AID.Detail.descriptionEditor]
        XCTAssertTrue(editor.waitForExistence(timeout: 5))
        let initialHeight = editor.frame.height
        editor.click()
        keyStroke("a", modifiers: [.command])
        let draft = "updated description\n" + (1 ... 24).map { "Draft line \($0)." }.joined(separator: "\n")
        paste(draft)
        XCTAssertGreaterThan(editor.frame.height, initialHeight + 40)
        let toggle = app.buttons[AID.Detail.descriptionExpansion]
        XCTAssertTrue(toggle.waitForExistence(timeout: 5))
        toggle.click()
        XCTAssertTrue(app.windows.firstMatch.frame.contains(editor.frame))
        XCTAssertTrue(app.buttons["Save"].isHittable)
        editor.click()
        keyStroke("a", modifiers: [.command])
        paste("Small draft")
        XCTAssertEqual(editor.frame.height, initialHeight, accuracy: 2)
        XCTAssertFalse(toggle.exists)
        app.buttons["Cancel"].click()
        XCTAssertFalse(editor.exists)
        XCTAssertFalse(toggle.exists)

        app.buttons["Edit"].click()
        editor.click()
        keyStroke("a", modifiers: [.command])
        paste(draft)
        app.buttons["Save"].click()
        XCTAssertFalse(editor.exists)
        selectChange("multiline description", in: app)
        selectChange("updated description", in: app)
        let saved = app.scrollViews[AID.Detail.description].staticTexts.firstMatch
        XCTAssertTrue((saved.value as? String ?? saved.label).contains("Draft line 24."))
    }

    private func selectChange(_ message: String, in app: XCUIApplication) {
        let row = dagRows(of: app).matching(NSPredicate(format: "value CONTAINS %@", message)).firstMatch
        XCTAssertTrue(row.waitForExistence(timeout: 10), "Change \(message) missing")
        row.click()
        XCTAssertTrue(
            app.scrollViews[AID.Detail.description].staticTexts
                .matching(NSPredicate(format: "value CONTAINS %@ OR label CONTAINS %@", message, message))
                .firstMatch.waitForExistence(timeout: 10),
            "Description for \(message) never loaded"
        )
    }
}
