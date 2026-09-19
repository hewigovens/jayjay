import XCTest

final class DescriptionHeightScene: SceneBase {
    override class var fixtureName: String {
        "description-height"
    }

    func testDescriptionFitsContentAndScrollsAboveCap() throws {
        let app = try XCTUnwrap(app)
        select("Short description", in: app)
        let description = app.descendants(matching: .any)[AID.Detail.description].firstMatch
        let short = description.frame.height
        let toggle = app.buttons[AID.Detail.descriptionExpansion]
        XCTAssertTrue(toggle.waitForExistence(timeout: 5), "The toggle also expands the metadata, so it is always offered")
        XCTAssertGreaterThan(short, 0)
        XCTAssertLessThan(short, 32)

        select("Multiline description", in: app)
        XCTAssertGreaterThan(description.frame.height, short * 2)
        XCTAssertLessThan(description.frame.height, 80)

        select("Wrapped description", in: app)
        XCTAssertGreaterThan(description.frame.height, short)

        select("Long description", in: app)
        let file = app.descendants(matching: .any)[AID.FileList.row("wip1.txt")].firstMatch
        XCTAssertTrue(file.waitForExistence(timeout: 5))
        file.click()
        let diff = app.textViews[AID.Diff.text].firstMatch
        XCTAssertTrue(diff.waitForExistence(timeout: 10))
        let scroll = app.scrollViews[AID.Detail.descriptionBody]
        XCTAssertEqual(scroll.frame.height, 75, accuracy: 1)
        let title = app.staticTexts[AID.Detail.descriptionTitle]
        let titleFrame = title.frame
        XCTAssertTrue(scroll.exists, "Long descriptions must scroll")
        let content = scroll.textViews.firstMatch
        let compact = XCTAttachment(screenshot: app.windows.firstMatch.screenshot())
        compact.name = "Compact description"
        compact.lifetime = .keepAlways
        add(compact)
        let beforeScroll = content.frame.minY
        scroll.scroll(byDeltaX: 0, deltaY: -3000)
        XCTAssertLessThan(content.frame.minY, beforeScroll)
        XCTAssertEqual(title.frame.minY, titleFrame.minY, accuracy: 1)
        XCTAssertEqual(scroll.frame.height, 75, accuracy: 1)

        XCTAssertTrue(toggle.waitForExistence(timeout: 5))
        toggle.click()
        let expandedHeight = scroll.frame.height
        let pane = app.descendants(matching: .any)[AID.Detail.pane].firstMatch
        XCTAssertTrue(pane.exists)
        let cap = max(160, pane.frame.height * 0.3)
        XCTAssertLessThanOrEqual(expandedHeight, cap + 2)
        XCTAssertGreaterThan(expandedHeight, cap - 17)
        XCTAssertTrue(file.isHittable)
        XCTAssertTrue(diff.isHittable)
        XCTAssertGreaterThanOrEqual(diff.frame.minY, description.frame.maxY)
        let expanded = XCTAttachment(screenshot: app.windows.firstMatch.screenshot())
        expanded.name = "Expanded description with visible diff"
        expanded.lifetime = .keepAlways
        add(expanded)
        let expandedScroll = content.frame.minY
        scroll.scroll(byDeltaX: 0, deltaY: -3000)
        XCTAssertLessThan(content.frame.minY, expandedScroll)
        XCTAssertEqual(title.frame.minY, titleFrame.minY, accuracy: 1)
        XCTAssertEqual(scroll.frame.height, expandedHeight, accuracy: 1)
        XCTAssertEqual(toggle.label, "Collapse description")
        toggle.click()
        XCTAssertEqual(scroll.frame.height, 75, accuracy: 1)
        XCTAssertEqual(content.frame.minY, scroll.frame.minY, accuracy: 1)
        toggle.click()
        XCTAssertEqual(scroll.frame.height, expandedHeight, accuracy: 1)
        select("Short description", in: app)
        XCTAssertEqual(description.frame.height, short, accuracy: 1)
        XCTAssertTrue(toggle.exists)
        select("Long description", in: app)
        XCTAssertEqual(scroll.frame.height, 75, accuracy: 1)
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
