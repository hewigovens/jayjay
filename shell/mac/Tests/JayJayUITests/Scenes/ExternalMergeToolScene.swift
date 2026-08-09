import XCTest

final class ExternalMergeToolScene: ExternalToolSceneBase {
    override class var additionalLaunchArguments: [String] {
        let root = fixtureRoot.appendingPathComponent("external-tool/merge", isDirectory: true)
        return [
            "tool", "merge",
            root.appendingPathComponent("left.swift").path,
            root.appendingPathComponent("base.swift").path,
            root.appendingPathComponent("right.swift").path,
            root.appendingPathComponent("output.swift").path,
            "Sources/ConflictSample.swift", "7"
        ]
    }

    func testJjMergeToolCanUseASideAndSaveTheOutput() throws {
        let app = try XCTUnwrap(app)
        let merge = app.descendants(matching: .any)[AID.ExternalTool.merge]
        XCTAssertTrue(merge.waitForExistence(timeout: 10), "External merge tool did not appear")

        let useFirstRightHunk = app.descendants(matching: .any)[AID.Conflict.hunkUse(0, "right")]
        XCTAssertTrue(useFirstRightHunk.waitForExistence(timeout: 5), "External merge did not reuse the hunk resolver")

        let rawMode = app.descendants(matching: .any)[AID.Conflict.editorRaw]
        rawMode.click()
        let result = app.descendants(matching: .any)[AID.Conflict.editorResult]
        XCTAssertTrue(result.waitForExistence(timeout: 5), "External merge did not reuse the raw editor")
        app.descendants(matching: .any)[AID.Conflict.editorHunks].click()
        XCTAssertTrue(useFirstRightHunk.waitForExistence(timeout: 5), "External merge did not return to hunk mode")
        useFirstRightHunk.click()

        var useSecondRightHunk = app.descendants(matching: .any)[AID.Conflict.hunkUse(1, "right")]
        XCTAssertFalse(useSecondRightHunk.isHittable, "The expanded second hunk should begin below the visible result pane")
        let hunkList = app.scrollViews[AID.Conflict.editorHunkList]
        XCTAssertTrue(hunkList.waitForExistence(timeout: 5), "External merge hunk list did not appear")
        // Keep the gesture in the list padding so the embedded native diff cannot consume it.
        let scrollPoint = hunkList.coordinate(withNormalizedOffset: CGVector(dx: 0.005, dy: 0.75))
        for _ in 0 ..< 8 where !useSecondRightHunk.isHittable {
            scrollPoint.scroll(byDeltaX: 0, deltaY: -500)
            useSecondRightHunk = app.descendants(matching: .any)[AID.Conflict.hunkUse(1, "right")]
        }
        XCTAssertTrue(useSecondRightHunk.isHittable, "External merge hunk list did not scroll to the second hunk")
        useSecondRightHunk.click()

        let save = app.buttons[AID.Conflict.editorSave]
        XCTAssertTrue(save.waitForExistence(timeout: 5), "Save action did not appear")
        XCTAssertTrue(save.isEnabled, "Per-hunk merge result should be saveable")
        save.click()
        XCTAssertTrue(app.wait(for: .notRunning, timeout: 10), "External merge tool did not finish")

        let output = Self.fixtureRoot
            .appendingPathComponent("external-tool/merge/output.swift")
        let saved = try String(contentsOf: output, encoding: .utf8)
        let right = Self.fixtureRoot
            .appendingPathComponent("external-tool/merge/right.swift")
        XCTAssertEqual(saved, try String(contentsOf: right, encoding: .utf8))
    }
}
