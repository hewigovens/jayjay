import XCTest

final class ExternalEditToolScene: ExternalToolSceneBase {
    override class var additionalLaunchArguments: [String] {
        let root = fixtureRoot.appendingPathComponent("external-tool", isDirectory: true)
        return [
            "tool", "edit",
            root.appendingPathComponent("edit-left", isDirectory: true).path,
            root.appendingPathComponent("edit-right", isDirectory: true).path
        ]
    }

    func testJjEditToolAppliesTheSelectedResult() throws {
        let app = try XCTUnwrap(app)
        let editor = app.descendants(matching: .any)[AID.ExternalTool.diff]
        XCTAssertTrue(editor.waitForExistence(timeout: 10), "External edit tool did not appear")
        XCTAssertTrue(app.staticTexts["Edit Diff"].exists)

        let toggle = app.buttons[AID.ExternalTool.fileToggle("file.txt")]
        XCTAssertTrue(toggle.waitForExistence(timeout: 5), "External edit file toggle did not appear")
        toggle.click()
        clickCenter(
            app.buttons[AID.ExternalTool.save],
            message: "External edit Done action did not appear"
        )
        XCTAssertTrue(app.wait(for: .notRunning, timeout: 10), "External edit tool did not finish")

        let output = Self.fixtureRoot
            .appendingPathComponent("external-tool/edit-right/file.txt")
        XCTAssertEqual(try String(contentsOf: output, encoding: .utf8), "before edit\n")
        let attributes = try FileManager.default.attributesOfItem(atPath: output.path)
        let permissions = try XCTUnwrap(attributes[.posixPermissions] as? NSNumber)
        XCTAssertEqual(permissions.intValue & 0o111, 0, "Discarding the edit should restore the left executable mode")
    }
}
