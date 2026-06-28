import XCTest

final class ReviewNotesScene: SceneBase {
    /// Dedicated fixture: adding and resolving review notes writes shared review state.
    override class var fixtureName: String {
        "simple-review-notes"
    }

    func testAddAndResolveReviewNoteFromGutterMarker() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let file = app.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier == %@", AID.FileList.row("scoring.swift")))
            .firstMatch
        clickCenter(file, timeout: 5, message: "scoring.swift row missing")

        let diff = app.descendants(matching: .any)[AID.Diff.section]
        XCTAssertTrue(diff.waitForExistence(timeout: 5), "Diff section did not appear")

        let add = openFirstChangeGroupMenu(app, expecting: "Add Review Note")
        XCTAssertTrue(add.waitForExistence(timeout: 5), "\"Add Review Note\" menu item missing")
        add.click()

        let body = app.textViews[AID.ReviewNote.body]
        XCTAssertTrue(body.waitForExistence(timeout: 5), "Review note editor did not appear")
        paste("Check this added line\nreview the loop below")

        let addNote = app.buttons["Add Note"]
        XCTAssertTrue(addNote.waitForExistence(timeout: 3), "\"Add Note\" button missing")
        addNote.click()

        // The count badge is a filter button, so match by identifier across element types.
        let activeCount = app.descendants(matching: .any)[AID.ReviewNote.activeCount(1)]
        XCTAssertTrue(activeCount.waitForExistence(timeout: 5), "Active note count did not update")
        let fileNoteVisible = NSPredicate { _, _ in
            file.value as? String == "1 review note"
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: fileNoteVisible, object: nil)], timeout: 5),
            .completed,
            "File row did not show its review-note count"
        )

        // Both body lines render as rows embedded in the diff text, expanding the view below the anchored line.
        let diffText = app.textViews[AID.Diff.text]
        let noteEmbedded = NSPredicate { _, _ in
            let text = diffText.value as? String ?? ""
            return text.contains("Check this added line") && text.contains("review the loop below")
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: noteEmbedded, object: nil)], timeout: 5),
            .completed,
            "Note body was not embedded in the diff view"
        )

        let edit = openFirstChangeGroupMenu(app, expecting: "Edit Review Note")
        XCTAssertTrue(edit.waitForExistence(timeout: 5), "\"Edit Review Note\" menu item missing")
        XCTAssertFalse(app.menuItems["Add Review Note"].exists, "\"Add Review Note\" should be hidden for noted lines")
        keyStroke(.escape)

        clickFirstChangeGroupMarker(app)

        let resolve = app.buttons["Resolve"]
        XCTAssertTrue(resolve.waitForExistence(timeout: 3), "\"Resolve\" popover action missing")
        resolve.click()

        let countCleared = NSPredicate { _, _ in
            !app.descendants(matching: .any)[AID.ReviewNote.activeCount(1)].exists
                && (file.value as? String ?? "").isEmpty
                && !(diffText.value as? String ?? "").contains("Check this added line")
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: countCleared, object: nil)], timeout: 5),
            .completed,
            "Active note count and embedded rows did not clear after resolve"
        )
    }

    /// The review context can lag the gutter's first render on cold CI runners; dismiss the incomplete menu and retry once.
    private func openFirstChangeGroupMenu(_ app: XCUIApplication, expecting title: String) -> XCUIElement {
        let item = app.menuItems[title]
        firstChangeGroupColumn(app).rightClick()
        if !item.waitForExistence(timeout: 3) {
            keyStroke(.escape)
            firstChangeGroupColumn(app).rightClick()
        }
        return item
    }

    private func clickFirstChangeGroupMarker(_ app: XCUIApplication) {
        // The note column sits right of the three-space group column (8pt inset + ~21pt); x=36 lands inside it across plausible gutter font sizes, where x=12 would toggle review state instead.
        gutterCoordinate(app, x: 36).click()
    }

    private func firstChangeGroupColumn(_ app: XCUIApplication) -> XCUICoordinate {
        gutterCoordinate(app, x: 12)
    }

    private func gutterCoordinate(_ app: XCUIApplication, x: CGFloat) -> XCUICoordinate {
        let gutter = app.textViews[AID.Diff.gutter]
        XCTAssertTrue(gutter.waitForExistence(timeout: 5), "Diff gutter did not appear")
        return gutter
            .coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0))
            .withOffset(CGVector(dx: x, dy: 12))
    }
}
