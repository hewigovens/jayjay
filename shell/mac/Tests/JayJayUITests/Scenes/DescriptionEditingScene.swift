import XCTest

final class DescriptionEditingScene: SceneBase {
    override class var fixtureName: String {
        "description-editing"
    }

    func testDescriptionDialogSavesAndCancelsWithoutReplacingWorkingCopyDraft() throws {
        let app = try XCTUnwrap(app)
        let draftSummary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(draftSummary.waitForExistence(timeout: 10))
        draftSummary.click()
        paste("Keep WIP summary")
        let draftBody = app.textViews[AID.CommitBox.draft]
        draftBody.click()
        paste("Keep WIP body")

        selectEditable(in: app)
        let preview = app.descendants(matching: .any)[AID.Detail.description].firstMatch
        let previewHeight = preview.frame.height
        let previewTitle = preview.staticTexts[AID.Detail.descriptionTitle]
        let previewBody = preview.scrollViews[AID.Detail.descriptionBody].textViews.firstMatch
        let initialSummary = try XCTUnwrap(previewTitle.value as? String)
        let initialBody = try XCTUnwrap(previewBody.value as? String)
        XCTAssertGreaterThan(initialSummary.count, 200)
        XCTAssertGreaterThan(app.buttons["Edit description"].frame.minX, preview.frame.minX)
        XCTAssertLessThanOrEqual(app.buttons["Edit description"].frame.maxY, preview.frame.maxY)
        let previewScreenshot = XCTAttachment(screenshot: preview.screenshot())
        previewScreenshot.name = "Edit beside wrapped title"
        previewScreenshot.lifetime = .keepAlways
        add(previewScreenshot)
        app.buttons["Edit description"].click()
        let sheet = app.sheets.firstMatch
        XCTAssertTrue(sheet.waitForExistence(timeout: 5))
        let summary = sheet.textFields[AID.CommitBox.summary]
        let body = sheet.textViews[AID.CommitBox.draft]
        XCTAssertEqual(summary.value as? String, initialSummary)
        XCTAssertEqual(body.value as? String, "    Editable body")
        let screenshot = XCTAttachment(screenshot: sheet.screenshot())
        screenshot.name = "Long summary initial caret"
        screenshot.lifetime = .keepAlways
        add(screenshot)
        paste(" appended")
        XCTAssertEqual(summary.value as? String, initialSummary + " appended")
        replace(summary, with: "Cancelled summary")
        replace(body, with: "Cancelled body")
        sheet.buttons["Cancel"].click()
        XCTAssertEqual(preview.frame.height, previewHeight, accuracy: 1)

        app.buttons["Edit description"].click()
        XCTAssertTrue(sheet.waitForExistence(timeout: 5))
        XCTAssertEqual(summary.value as? String, initialSummary)
        XCTAssertEqual(body.value as? String, "    Editable body")
        sheet.buttons["Save"].click()
        XCTAssertTrue(sheet.waitForNonExistence(timeout: 10))
        XCTAssertEqual(previewTitle.value as? String, initialSummary)
        XCTAssertEqual(previewBody.value as? String, initialBody)
        app.buttons["Edit description"].click()
        XCTAssertTrue(sheet.waitForExistence(timeout: 5))
        replace(summary, with: "Updated summary")
        replace(body, with: "Updated body\nSecond body line")
        sheet.buttons["Save"].click()
        let saved = preview.staticTexts.matching(NSPredicate(format: "value == %@", "Updated summary")).firstMatch
        XCTAssertTrue(saved.waitForExistence(timeout: 10))
        XCTAssertEqual(previewBody.value as? String, "Updated body\nSecond body line")

        app.buttons["Edit description"].click()
        XCTAssertTrue(sheet.waitForExistence(timeout: 5))
        replace(summary, with: "")
        replace(body, with: "")
        sheet.buttons["Save"].click()
        let addDescription = app.buttons["Add description"]
        XCTAssertTrue(addDescription.waitForExistence(timeout: 10))
        XCTAssertFalse(app.buttons["Edit description"].exists)
        addDescription.click()
        XCTAssertTrue(sheet.waitForExistence(timeout: 5))
        XCTAssertEqual(summary.value as? String ?? "", "")
        XCTAssertEqual(body.value as? String ?? "", "")
        replace(summary, with: "Added description")
        sheet.buttons["Save"].click()
        XCTAssertTrue(preview.staticTexts.matching(NSPredicate(format: "value == %@", "Added description")).firstMatch.waitForExistence(timeout: 10))
        XCTAssertTrue(app.buttons["Edit description"].exists)

        dagRows(of: app).element(boundBy: 0).coordinate(withNormalizedOffset: CGVector(dx: 0.8, dy: 0.65)).click()
        XCTAssertTrue(draftSummary.waitForExistence(timeout: 5))
        XCTAssertEqual(draftSummary.value as? String, "Keep WIP summary")
        XCTAssertEqual(draftBody.value as? String, "Keep WIP body")
    }

    private func selectEditable(in app: XCUIApplication) {
        let row = dagRows(of: app).matching(NSPredicate(format: "value CONTAINS %@", "Editable summary")).firstMatch
        XCTAssertTrue(row.waitForExistence(timeout: 10))
        row.coordinate(withNormalizedOffset: CGVector(dx: 0.8, dy: 0.65)).click()
        XCTAssertTrue(app.buttons["Edit description"].waitForExistence(timeout: 5))
    }

    private func replace(_ field: XCUIElement, with text: String) {
        field.click()
        field.typeKey("a", modifierFlags: .command)
        if text.isEmpty {
            field.typeKey(.delete, modifierFlags: [])
        } else {
            paste(text)
        }
    }
}
