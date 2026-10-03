import XCTest

final class SummaryTypingScene: SceneBase {
    override class var fixtureName: String {
        "summary-typing"
    }

    override class var additionalLaunchArguments: [String] {
        ["-jayjay.fontFamily", "system", "-jayjay.fontSize", "10", "-jayjay.sidebarWidth", "360"]
    }

    func testSummaryTypingThroughOverflowAndFocusChanges() throws {
        let app = try XCTUnwrap(app)
        let summary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(summary.waitForExistence(timeout: 10))
        XCTAssertTrue(app.descendants(matching: .any)[AID.FileList.row("xx.md")].waitForExistence(timeout: 5), "@ must contain xx.md")
        summary.click()
        keyStroke("a", modifiers: [.command])
        keyStroke(.delete)

        let text = "this is some text hello world hello world some more text hello world"
        for character in text {
            keyStroke(String(character))
        }
        XCTAssertEqual(summary.value as? String, text)
        let details = app.textViews[AID.CommitBox.draft]
        details.click()
        XCTAssertEqual(summary.value as? String, text)
        let startClickInset: CGFloat = 1
        summary.coordinate(withNormalizedOffset: CGVector(dx: startClickInset / summary.frame.width, dy: 0.5)).click()
        var prefix = ""
        for character in "abcd" {
            keyStroke(String(character))
            prefix.append(character)
            XCTAssertEqual(summary.value as? String, prefix + text, "Typing must stay at the clicked insertion point")
        }
        let attachment = XCTAttachment(screenshot: summary.screenshot())
        attachment.name = "Summary after refocusing at the start and typing"
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
