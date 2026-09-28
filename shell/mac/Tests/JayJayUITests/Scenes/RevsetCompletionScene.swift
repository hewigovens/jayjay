import XCTest

final class RevsetCompletionScene: SceneBase {
    override class var fixtureName: String {
        "revset-completion"
    }

    func testCompletingInTheRevsetBarReplacesTheWholeSymbol() throws {
        let app = try XCTUnwrap(app)
        let bar = app.buttons[AID.Toolbar.revsetBar]
        XCTAssertTrue(bar.waitForExistence(timeout: 10))
        bar.click()
        let field = app.descendants(matching: .any)[AID.Toolbar.revsetField].firstMatch
        XCTAssertTrue(field.waitForExistence(timeout: 5))

        for (typed, completed) in [
            ("anc", "ancestors("),
            ("wip-1", "wip-1-extra"),
            ("fix-a", "\"fix-a|b\"")
        ] {
            keyStroke("a", modifiers: [.command])
            paste(typed)
            XCTAssertEqual(field.value as? String, typed)
            XCTAssertTrue(
                app.buttons[AID.Picker.row("completion-\(completed)")].waitForExistence(timeout: 5),
                "typing \(typed) opens the list"
            )
            keyStroke(.downArrow)
            keyStroke(.return)
            XCTAssertEqual(field.value as? String, completed, "completing \(typed)")
        }

        keyStroke("a", modifiers: [.command])
        paste("main")
        keyStroke(.leftArrow)
        keyStroke(.leftArrow)
        keyStroke(.downArrow)
        keyStroke(.return)
        XCTAssertEqual(field.value as? String, "main", "completing inside a symbol replaces all of it")

        keyStroke("a", modifiers: [.command])
        paste("au")
        XCTAssertTrue(app.buttons[AID.Picker.row("completion-author(")].waitForExistence(timeout: 5))
        keyStroke(.escape)
        XCTAssertFalse(app.buttons[AID.Picker.row("completion-author(")].exists)
        XCTAssertEqual(field.value as? String, "au", "Escape closes the list before the edit")

        keyStroke("a", modifiers: [.command])
        paste("@ | main")
        keyStroke(.return)
        XCTAssertTrue(bar.waitForExistence(timeout: 5))
        XCTAssertEqual(bar.value as? String, "@ | main", "Return applies the revset while no row is picked")
    }

    func testRevsetPanelCompletesAboveItsRowsAndStillAppliesThem() throws {
        let app = try XCTUnwrap(app)
        let presets = app.buttons["Presets, bookmarks and recent revsets"]
        XCTAssertTrue(presets.waitForExistence(timeout: 10))
        presets.click()
        let field = app.descendants(matching: .any)[AID.Toolbar.revsetField].firstMatch
        XCTAssertTrue(field.waitForExistence(timeout: 5))
        field.click()

        paste("@ | mai")
        XCTAssertTrue(app.buttons[AID.Picker.row("completion-main")].waitForExistence(timeout: 5))
        keyStroke(.downArrow)
        keyStroke(.return)
        XCTAssertEqual(field.value as? String, "@ | main", "a completion fills the field instead of applying")

        keyStroke("a", modifiers: [.command])
        paste("ma")
        XCTAssertTrue(app.buttons[AID.Picker.row("bookmark-main")].waitForExistence(timeout: 5))
        keyStroke(.downArrow)
        keyStroke(.return)
        let bar = app.buttons[AID.Toolbar.revsetBar]
        XCTAssertTrue(bar.waitForExistence(timeout: 5))
        let revset = try XCTUnwrap(bar.value as? String)
        XCTAssertTrue(
            revset.contains("bookmarks(exact:\"main\")"),
            "Down and Return activate the bookmark row, got \(revset)"
        )
    }
}
