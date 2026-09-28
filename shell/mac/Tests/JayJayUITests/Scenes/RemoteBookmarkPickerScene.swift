import XCTest

final class RemoteBookmarkPickerScene: SceneBase {
    override class var fixtureName: String {
        "remote-bookmarks"
    }

    func testBrowseRemoteHistoryWithoutTracking() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10))
        let bar = app.buttons[AID.Toolbar.revsetBar]
        let defaultRevset = try XCTUnwrap(bar.value as? String)
        let picker = app.buttons.matching(NSPredicate(format: "label BEGINSWITH 'Bookmarks'")).firstMatch
        XCTAssertTrue(picker.waitForExistence(timeout: 5))
        picker.click()
        let originRow = app.buttons[AID.Picker.row("remote-bookmark-10:other-workorigin")]
        XCTAssertTrue(originRow.waitForExistence(timeout: 5), "Browsing must leave this bookmark remote-only")
        originRow.click()
        XCTAssertTrue(rows.firstMatch.wait(for: \.isSelected, toEqual: false, timeout: 5), "A bookmark row did not select its change")
        XCTAssertEqual(bar.value as? String, defaultRevset, "Selecting a shown bookmark must not change the filter")

        for (remote, count) in [("origin", 2), ("upstream", 1)] {
            bar.click()
            let field = app.textFields[AID.Toolbar.revsetField]
            XCTAssertTrue(field.waitForExistence(timeout: 5))
            keyStroke("a", modifiers: [.command])
            paste("other-work@\(remote)")
            keyStroke(.return)
            XCTAssertTrue(field.waitForNonExistence(timeout: 5))
            let expected = NSPredicate { _, _ in rows.count == count }
            let ready = XCTNSPredicateExpectation(predicate: expected, object: nil)
            XCTAssertEqual(XCTWaiter().wait(for: [ready], timeout: 10), .completed)
        }
        picker.click()
        XCTAssertTrue(app.buttons[AID.Picker.row("remote-bookmark-10:other-workorigin")].waitForExistence(timeout: 5))
        XCTAssertTrue(app.buttons[AID.Picker.row("remote-bookmark-10:other-workupstream")].exists)
        XCTAssertTrue(app.buttons[AID.Picker.row("remote-bookmark-12:deleted-workupstream")].exists)
        XCTAssertFalse(app.buttons[AID.Picker.row("remote-bookmark-12:deleted-workorigin")].exists)
        keyStroke(.escape)
    }
}
