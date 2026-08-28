import XCTest

/// The `sync-cancel` fixture routes `git` to a script that never returns, so Pull stays in flight until canceled.
final class PullCancelScene: SceneBase {
    override class var fixtureName: String {
        "sync-cancel"
    }

    func testCancelPullFromToolbar() throws {
        let app = try XCTUnwrap(app)
        let pull = app.buttons[AID.Toolbar.pull]
        XCTAssertTrue(pull.waitForExistence(timeout: 10), "Pull toolbar button missing")

        pull.click()
        XCTAssertTrue(
            pull.wait(for: \.label, toEqual: "Cancel Pull", timeout: 10),
            "Pull did not switch to Cancel while in flight"
        )

        pull.click()
        XCTAssertTrue(app.staticTexts["Pull canceled"].waitForExistence(timeout: 10), "Cancel toast missing")
        XCTAssertTrue(pull.wait(for: \.label, toEqual: "Pull", timeout: 10), "Pull button did not return to idle")
    }
}
