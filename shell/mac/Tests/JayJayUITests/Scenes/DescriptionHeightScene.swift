import XCTest

final class DescriptionHeightScene: SceneBase {
    override class var fixtureName: String {
        "description-height"
    }

    override class var additionalLaunchArguments: [String] {
        ["fontSize", "interfaceFontSize", "fontFamily", "interfaceFontFamily"].flatMap { ["-jayjay.\($0)", ""] }
    }

    func testResetZoomRestoresFirstLaunchSizeFromShortcutAndPalette() throws {
        let app = try XCTUnwrap(app)
        select("Short description", in: app)
        let title = app.staticTexts[AID.Detail.descriptionTitle].firstMatch
        let defaultHeight = title.frame.height
        XCTAssertGreaterThan(defaultHeight, 0)

        keyStroke("-", modifiers: .command)
        let smaller = NSPredicate { _, _ in title.frame.height < defaultHeight }
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: smaller, object: nil)], timeout: 3), .completed)
        keyStroke("0", modifiers: .command)
        let restored = NSPredicate { _, _ in abs(title.frame.height - defaultHeight) < 0.5 }
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: restored, object: nil)], timeout: 3), .completed)

        keyStroke("-", modifiers: .command)
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: smaller, object: nil)], timeout: 3), .completed)
        keyStroke("p", modifiers: [.command, .shift])
        let field = app.textFields[AID.Palette.textField]
        XCTAssertTrue(field.waitForExistence(timeout: 5))
        field.click()
        paste("Reset Zoom")
        let reset = app.element(AID.Palette.item("Reset Zoom")).firstMatch
        XCTAssertTrue(reset.waitForExistence(timeout: 3))
        reset.click()
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: restored, object: nil)], timeout: 3), .completed)
    }

    func testDescriptionFitsContentAndCapsLongBodies() throws {
        let app = try XCTUnwrap(app)
        select("Short description", in: app)
        let description = app.element(AID.Detail.description).firstMatch
        let short = description.frame.height
        let toggle = app.buttons[AID.Detail.descriptionExpansion]
        XCTAssertTrue(toggle.waitForExistence(timeout: 5), "The toggle also expands the metadata, so it is always offered")
        XCTAssertGreaterThan(short, 0)

        select("Multiline description", in: app)
        XCTAssertGreaterThan(description.frame.height, short)

        select("Wrapped description", in: app)
        XCTAssertGreaterThan(description.frame.height, short)

        select("Long description", in: app)
        let file = app.element(AID.FileList.row("wip1.txt")).firstMatch
        XCTAssertTrue(file.waitForExistence(timeout: 5))
        let headerTitle = app.staticTexts[AID.Detail.descriptionTitle].firstMatch
        XCTAssertLessThan(headerTitle.frame.minX, file.frame.maxX, "The change header does not span the file list")
        XCTAssertGreaterThan(file.frame.minY, headerTitle.frame.maxY, "The file list starts beside the change header instead of below it")
        file.click()
        let diff = app.textViews[AID.Diff.text].firstMatch
        XCTAssertTrue(diff.waitForExistence(timeout: 10))
        let scroll = app.scrollViews[AID.Detail.descriptionBody]
        let compactHeight = scroll.frame.height
        XCTAssertGreaterThan(compactHeight, 0)
        XCTAssertLessThanOrEqual(compactHeight, file.frame.height)
        XCTAssertTrue(scroll.exists, "Long descriptions must scroll")
        capture(app.windows.firstMatch, name: "Compact description")
        XCTAssertTrue(toggle.waitForExistence(timeout: 5))
        toggle.click()
        let expandedHeight = scroll.frame.height
        let pane = app.element(AID.Detail.pane).firstMatch
        XCTAssertTrue(pane.exists)
        XCTAssertGreaterThan(expandedHeight, compactHeight)
        XCTAssertLessThan(expandedHeight, pane.frame.height)
        XCTAssertTrue(file.isHittable)
        XCTAssertTrue(diff.isHittable)
        XCTAssertGreaterThanOrEqual(diff.frame.minY, description.frame.maxY)
        capture(app.windows.firstMatch, name: "Expanded description with visible diff")
        XCTAssertEqual(toggle.label, "Collapse description")
        toggle.click()
        XCTAssertEqual(scroll.frame.height, compactHeight, accuracy: 1)
        toggle.click()
        XCTAssertEqual(scroll.frame.height, expandedHeight, accuracy: 1)
        select("Short description", in: app)
        XCTAssertEqual(description.frame.height, short, accuracy: 1)
        XCTAssertTrue(toggle.exists)
        select("Long description", in: app)
        XCTAssertEqual(scroll.frame.height, compactHeight, accuracy: 1)
        XCTAssertEqual(toggle.label, "Expand description")
    }

    private func select(_ subject: String, in app: XCUIApplication) {
        let row = dagRows(of: app).matching(NSPredicate(format: "value CONTAINS %@", subject)).firstMatch
        XCTAssertTrue(row.waitForExistence(timeout: 10))
        row.coordinate(withNormalizedOffset: CGVector(dx: 0.8, dy: 0.65)).click()
        let text = app.staticTexts.matching(identifier: AID.Detail.descriptionTitle)
            .matching(NSPredicate(format: "value CONTAINS %@", subject)).firstMatch
        XCTAssertTrue(text.waitForExistence(timeout: 10), "Description for \(subject) did not load")
    }
}
