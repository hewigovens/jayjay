import XCTest

final class KeyboardPaneFocusScene: SceneBase {
    func testTabCyclesPanesAndEscapeLeavesTheFilterField() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10), "DAG never populated")
        rows.firstMatch.click()
        let firstFile = app.descendants(matching: .any)[AID.FileList.row("wip1.txt")]
        let secondFile = app.descendants(matching: .any)[AID.FileList.row("wip2.txt")]
        XCTAssertTrue(firstFile.waitForExistence(timeout: 10), "Detail never loaded the working copy")

        keyStroke(.tab)
        keyStroke(.downArrow)
        XCTAssertTrue(secondFile.wait(for: \.isSelected, toEqual: true, timeout: 5), "Tab did not hand navigation to the file list")
        XCTAssertTrue(rows.firstMatch.isSelected, "Down arrow moved the DAG selection instead of the file selection")

        keyStroke(.tab)
        keyStroke(.tab)
        keyStroke(.space)
        let filterField = app.descendants(matching: .any)[AID.FileList.filterField]
        XCTAssertTrue(filterField.waitForExistence(timeout: 5), "Space on the filter toggle did not open the filter")
        paste("wip2")
        XCTAssertTrue(firstFile.waitForNonExistence(timeout: 5), "The opened filter did not take typing")
        keyStroke(.escape)
        XCTAssertTrue(filterField.waitForNonExistence(timeout: 5), "Escape did not close the filter")
        XCTAssertTrue(firstFile.waitForExistence(timeout: 5), "Escape did not clear the filter")
        keyStroke(.upArrow)
        XCTAssertTrue(firstFile.wait(for: \.isSelected, toEqual: true, timeout: 5), "Escape did not return navigation to the file list")

        keyStroke(.tab, modifiers: [.shift])
        keyStroke(.tab, modifiers: [.shift])
        paste("Keyboard description")
        let description = app.textViews[AID.CommitBox.draft]
        XCTAssertEqual(description.value as? String, "Keyboard description")
        keyStroke(.tab, modifiers: [.shift])
        keyStroke("a", modifiers: [.command])
        paste("Keyboard summary")
        XCTAssertEqual(app.textFields[AID.CommitBox.summary].value as? String, "Keyboard summary")
        keyStroke(.tab)
        keyStroke(.tab)
        keyStroke(.downArrow)
        let featureFile = app.descendants(matching: .any)[AID.FileList.row("feature.txt")]
        XCTAssertTrue(featureFile.waitForExistence(timeout: 10), "Shift-Tab did not hand navigation back to the DAG")

        // File list, tree, filter, diff layout, edit description, edit diff, then the toolbar filter.
        for _ in 0 ..< 7 {
            keyStroke(.tab)
        }
        keyStroke(.space)
        let revset = app.textFields["Revset expression"]
        XCTAssertTrue(revset.waitForExistence(timeout: 5), "Space on the toolbar filter did not open the revset filter")
        keyStroke("a", modifiers: [.command])
        paste("all()")
        XCTAssertEqual(revset.value as? String, "all()", "The keyboard-opened revset filter did not take typing")
        for _ in 0 ..< 7 {
            keyStroke(.tab)
        }
        keyStroke(.downArrow)
        let helloFile = app.descendants(matching: .any)[AID.FileList.row("hello.txt")]
        XCTAssertTrue(helloFile.waitForExistence(timeout: 10), "Tab past the toolbar did not wrap around to the DAG")
    }

    func testDiffEditKeepsNavigationUntilItCloses() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10))
        rows.firstMatch.click()
        let open = app.buttons[AID.DiffEdit.open]
        XCTAssertTrue(open.waitForExistence(timeout: 10))
        open.click()
        let first = app.buttons[AID.DiffEdit.fileToggle("wip1.txt")]
        XCTAssertTrue(first.waitForExistence(timeout: 5))
        keyStroke(.tab)
        keyStroke(.downArrow)
        keyStroke(.return)
        XCTAssertEqual(first.value as? String, "collapsed")
        XCTAssertFalse(app.textFields["Revset expression"].exists)
        app.buttons[AID.DiffEdit.cancel].click()
        XCTAssertTrue(open.waitForExistence(timeout: 5))
        keyStroke(.tab)
        keyStroke(.downArrow)
        let second = app.descendants(matching: .any)[AID.FileList.row("wip2.txt")]
        XCTAssertTrue(second.wait(for: \.isSelected, toEqual: true, timeout: 5))
    }
}
