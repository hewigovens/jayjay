import XCTest

final class RemoteBookmarkPickerScene: SceneBase {
    override class var fixtureName: String {
        "remote-bookmarks"
    }

    func testBrowseRemoteHistoryWithoutTracking() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).firstMatch.waitForExistence(timeout: 10))
        for (remote, count) in [("origin", 2), ("upstream", 1)] {
            let picker = app.toolbars.buttons.matching(NSPredicate(format: "label BEGINSWITH 'Bookmarks'")).firstMatch
            XCTAssertTrue(picker.waitForExistence(timeout: 5))
            picker.click()
            let row = app.buttons[AID.Picker.row("remote-bookmark-10:other-work\(remote)")]
            XCTAssertTrue(row.waitForExistence(timeout: 5), "Browsing must leave this bookmark remote-only")
            let filter = app.textFields["Filter"].firstMatch
            XCTAssertTrue(filter.waitForExistence(timeout: 5))
            filter.click()
            paste("other-work@\(remote)")
            keyStroke(.return)
            XCTAssertTrue(row.waitForNonExistence(timeout: 5))
            let rows = dagRows(of: app)
            let expected = NSPredicate { _, _ in rows.count == count }
            let ready = XCTNSPredicateExpectation(predicate: expected, object: nil)
            XCTAssertEqual(XCTWaiter().wait(for: [ready], timeout: 10), .completed)
        }
        app.toolbars.buttons.matching(NSPredicate(format: "label BEGINSWITH 'Bookmarks'")).firstMatch.click()
        XCTAssertTrue(app.buttons[AID.Picker.row("remote-bookmark-10:other-workorigin")].waitForExistence(timeout: 5))
        XCTAssertTrue(app.buttons[AID.Picker.row("remote-bookmark-10:other-workupstream")].exists)
        XCTAssertTrue(app.buttons[AID.Picker.row("remote-bookmark-12:deleted-workupstream")].exists)
        XCTAssertFalse(app.buttons[AID.Picker.row("remote-bookmark-12:deleted-workorigin")].exists)
        keyStroke(.escape)
    }
}
