import XCTest

final class ConflictResolutionScene: SceneBase {
    override class var fixtureName: String {
        "conflict"
    }

    func testConflictFileShowsDiff() throws {
        let app = try XCTUnwrap(app)
        let file = fileRows(of: app)
            .matching(NSPredicate(format: "identifier == %@", AID.FileList.row("conflict.swift")))
            .firstMatch
        XCTAssertTrue(file.waitForExistence(timeout: 10), "Expected conflicted file row")
        file.click()

        let diff = app.descendants(matching: .any)[AID.Diff.section]
        XCTAssertTrue(diff.waitForExistence(timeout: 5), "Diff section did not appear")
    }

    func testConflictCanBeEditedWithoutOpeningAnotherTool() throws {
        let app = try XCTUnwrap(app)
        let edit = app.buttons[AID.Conflict.resolveInJayJay("conflict.swift")]
        clickCenter(edit, timeout: 10, message: "Expected Edit in JayJay action")

        // Preparation highlights five texts before the sheet opens; on a slow CI runner that outlasts a short fixed wait, so wait for the app's own loading state to clear.
        let preparing = app.descendants(matching: .any)[AID.Conflict.editorPreparing]
        XCTAssertTrue(preparing.waitForNonExistence(timeout: 60), "Conflict editor preparation did not finish")
        let modal = app.staticTexts[AID.Conflict.editorModal]
        XCTAssertTrue(modal.waitForExistence(timeout: 10), "Conflict editor modal did not appear")
        let useBase = app.buttons[AID.ExternalTool.useSource("base")]
        let useLeft = app.buttons[AID.ExternalTool.useSource("left")]
        let useRight = app.buttons[AID.ExternalTool.useSource("right")]
        XCTAssertFalse(useBase.exists, "Base should stay hidden until requested")
        XCTAssertTrue(useLeft.exists, "Left source should be visible by default")
        XCTAssertTrue(useRight.exists, "Right source should be visible by default")
        let baseVisibility = app.buttons[AID.ExternalTool.baseVisibility]
        XCTAssertTrue(baseVisibility.waitForExistence(timeout: 5), "Show Base action did not appear")
        baseVisibility.click()
        XCTAssertTrue(useBase.waitForExistence(timeout: 5), "Base source did not appear")
        XCTAssertFalse(useLeft.exists, "Base mode should replace the Left source")
        XCTAssertFalse(useRight.exists, "Base mode should replace the Right source")
        baseVisibility.click()
        XCTAssertTrue(useLeft.waitForExistence(timeout: 5), "Left source did not return")
        XCTAssertTrue(useRight.waitForExistence(timeout: 5), "Right source did not return")
        XCTAssertFalse(useBase.exists, "Base source should hide after returning")

        let rawMode = app.descendants(matching: .any)[AID.Conflict.editorRaw]
        XCTAssertTrue(rawMode.waitForExistence(timeout: 5), "Raw result mode did not appear")
        rawMode.click()
        let result = app.descendants(matching: .any)[AID.Conflict.editorResult]
        XCTAssertTrue(result.waitForExistence(timeout: 5), "Conflict result editor did not appear")
        let hunkMode = app.descendants(matching: .any)[AID.Conflict.editorHunks]
        hunkMode.click()
        XCTAssertTrue(result.waitForNonExistence(timeout: 5), "Raw editor did not switch back to hunks")
        let useRightHunk = app.buttons[AID.Conflict.hunkUse(0, "right")]
        XCTAssertTrue(useRightHunk.waitForExistence(timeout: 5), "Per-hunk Accept Right action did not appear")
        useRightHunk.click()

        let save = app.buttons[AID.Conflict.editorSave]
        XCTAssertTrue(save.isEnabled, "Hunk resolution should be saveable")
        save.click()
        XCTAssertTrue(modal.waitForNonExistence(timeout: 10), "Editor did not close after saving")
    }
}
