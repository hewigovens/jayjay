import XCTest

final class BookmarkManagerScene: SceneBase {
    override class var fixtureName: String {
        "remote-bookmarks"
    }

    func testFilterInDAG() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10))

        for (name, head, description, count) in [
            ("main", "bookmarks(exact:\"main\")", "add feature", 3),
            ("other-work@origin", "remote_bookmarks(exact:\"other-work\", exact:\"origin\")", "add hello", 2)
        ] {
            keyStroke("b", modifiers: [.command, .shift])
            let title = app.staticTexts["Bookmark Manager"]
            XCTAssertTrue(title.waitForExistence(timeout: 5), "Bookmark Manager did not open")
            let bookmark = app.sheets.staticTexts[name]
            XCTAssertTrue(bookmark.waitForExistence(timeout: 5))
            bookmark.rightClick()
            app.menuItems["Filter in DAG"].click()
            XCTAssertTrue(title.waitForNonExistence(timeout: 5))
            XCTAssertEqual(app.buttons[AID.Toolbar.revsetBar].value as? String, "\(head) | trunk()..\(head)")
            let filtered = NSPredicate { _, _ in
                rows.count == count && (rows.firstMatch.value as? String)?.contains(description) == true
            }
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: filtered, object: nil)], timeout: 10),
                .completed,
                "Filtering \(name) should show its target and stack"
            )
        }
    }
}
