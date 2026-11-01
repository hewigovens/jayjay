import XCTest

final class FormatPreviewScene: SceneBase {
    override static var fixtureName: String {
        "formats"
    }

    func testFormatsOpenInTheirDefaultViewAndReturnToSource() throws {
        let app = try XCTUnwrap(app)
        let rendered = app.webViews.firstMatch

        select("results.sarif", in: app)
        XCTAssertTrue(rendered.waitForExistence(timeout: 10), "SARIF should open rendered")
        app.buttons[AID.Diff.preview].click()
        XCTAssertTrue(rendered.waitForNonExistence(timeout: 5), "Preview should return SARIF to source")

        select("logo.svg", in: app)
        XCTAssertTrue(rendered.waitForExistence(timeout: 10), "SVG should open rendered")
        app.buttons[AID.Diff.preview].click()
        XCTAssertTrue(rendered.waitForNonExistence(timeout: 5), "Preview should return SVG to source")

        select("analysis.ipynb", in: app)
        XCTAssertTrue(app.buttons[AID.Diff.source].waitForExistence(timeout: 10), "Notebook should offer its source")
        XCTAssertFalse(rendered.exists, "Notebook should open as a text diff")
        app.buttons[AID.Diff.preview].click()
        XCTAssertTrue(rendered.waitForExistence(timeout: 5), "Preview should render the notebook")
        app.buttons[AID.Diff.source].click()
        XCTAssertTrue(rendered.waitForNonExistence(timeout: 5), "Source should leave the rendered notebook")
    }

    private func select(_ path: String, in app: XCUIApplication) {
        let row = app.element(AID.FileList.row(path))
        XCTAssertTrue(row.waitForExistence(timeout: 10), "\(path) missing from the file list")
        row.click()
    }
}
