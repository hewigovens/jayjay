import XCTest

final class ExternalMergeToolScene: ExternalToolSceneBase {
    override class var additionalLaunchArguments: [String] {
        let root = fixtureRoot.appendingPathComponent("external-tool", isDirectory: true)
        return [
            "tool", "merge",
            root.appendingPathComponent("merge-left.txt").path,
            root.appendingPathComponent("merge-base.txt").path,
            root.appendingPathComponent("merge-right.txt").path,
            root.appendingPathComponent("merge-output.txt").path
        ]
    }

    func testJjMergeToolWritesTheChosenSide() throws {
        let app = try XCTUnwrap(app)
        let resolver = app.descendants(matching: .any)[AID.ExternalTool.merge]
        XCTAssertTrue(resolver.waitForExistence(timeout: 10), "External merge tool did not appear")
        XCTAssertTrue(app.staticTexts["merge-output.txt"].exists, "External merge tool did not name the merged file")

        clickCenter(
            app.buttons[AID.ExternalTool.useSource("left")],
            message: "External merge Use Left action did not appear"
        )
        clickCenter(
            app.buttons[AID.Conflict.editorSave],
            message: "External merge Save action did not appear"
        )
        XCTAssertTrue(app.wait(for: .notRunning, timeout: 10), "External merge tool did not finish")

        let output = Self.fixtureRoot
            .appendingPathComponent("external-tool/merge-output.txt")
        XCTAssertEqual(try String(contentsOf: output, encoding: .utf8), "shared\nleft change\n")
    }
}
