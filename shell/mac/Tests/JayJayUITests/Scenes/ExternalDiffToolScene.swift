import XCTest

final class ExternalDiffToolScene: ExternalToolSceneBase {
    override class var additionalLaunchArguments: [String] {
        let root = fixtureRoot.appendingPathComponent("external-tool", isDirectory: true)
        return [
            "tool", "diff",
            root.appendingPathComponent("diff-left", isDirectory: true).path,
            root.appendingPathComponent("diff-right", isDirectory: true).path
        ]
    }

    func testJjDiffToolOpensReadOnlyComparison() throws {
        let app = try XCTUnwrap(app)
        let comparison = app.descendants(matching: .any)[AID.ExternalTool.diff]
        XCTAssertTrue(comparison.waitForExistence(timeout: 10), "External diff tool did not appear")
        XCTAssertTrue(app.staticTexts["Folder Comparison"].exists)
        XCTAssertTrue(app.staticTexts["file.txt"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.buttons["Done"].exists, "Read-only diff tool must not offer saving")
    }
}
