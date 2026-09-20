import XCTest

final class PullRequestImportScene: SceneBase {
    func testResolveErrorsStayInlineAndEscapeCloses() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).firstMatch.waitForExistence(timeout: 10), "DAG never populated")

        keyStroke("p", modifiers: [.command, .shift])
        let paletteField = app.textFields[AID.Palette.textField]
        XCTAssertTrue(paletteField.waitForExistence(timeout: 5))
        paletteField.click()
        paste("from Pull Request")
        let command = app.descendants(matching: .any)[AID.Palette.item("New Workspace from Pull Request…")]
        XCTAssertTrue(command.waitForExistence(timeout: 5), "Palette item missing")
        command.click()

        let sheet = app.sheets.firstMatch
        XCTAssertTrue(sheet.waitForExistence(timeout: 5), "Sheet did not appear")
        let urlField = sheet.textFields[AID.PullRequestImport.urlField]
        XCTAssertTrue(urlField.waitForExistence(timeout: 5), "URL field missing")
        let resolve = sheet.buttons[AID.PullRequestImport.resolveButton]
        XCTAssertTrue(resolve.waitForExistence(timeout: 5), "Resolve button missing")
        XCTAssertFalse(resolve.isEnabled, "Resolve must stay disabled while the URL is empty")

        urlField.click()
        paste("not a url")
        keyStroke(.return)
        let error = sheet.descendants(matching: .any)[AID.PullRequestImport.error]
        XCTAssertTrue(error.waitForExistence(timeout: 10), "Inline error did not appear")
        XCTAssertEqual(urlField.value as? String, "not a url", "Failed resolve must keep the entered URL")

        urlField.click()
        keyStroke("a", modifiers: [.command])
        paste("https://github.com/o/r/pull/1")
        keyStroke(.return)
        let originError = sheet.descendants(matching: .any).matching(
            NSPredicate(format: "identifier == %@ AND value CONTAINS %@", AID.PullRequestImport.error, "origin")
        )
        XCTAssertTrue(originError.firstMatch.waitForExistence(timeout: 10), "Origin error did not replace the parse error")

        keyStroke(.escape)
        XCTAssertTrue(sheet.waitForNonExistence(timeout: 5), "Escape must close the sheet")
    }
}
