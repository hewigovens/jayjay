import XCTest

final class FileEditorScene: SceneBase {
    override class var fixtureName: String {
        "file-editor"
    }

    func testWorkingCopyFileCanBeEditedAndSavedInJayJay() throws {
        let app = try XCTUnwrap(app)
        let file = fileRows(of: app)
            .matching(NSPredicate(format: "identifier == %@", AID.FileList.row("wip1.txt")))
            .firstMatch
        XCTAssertTrue(file.waitForExistence(timeout: 10), "Expected working-copy file row")
        file.rightClick()
        XCTAssertTrue(app.menuItems["Show in Finder"].waitForExistence(timeout: 3))
        XCTAssertFalse(app.menuItems["Edit File in JayJay"].exists)
        app.typeKey(.escape, modifierFlags: [])
        file.click()

        let open = app.buttons["Edit File"]
        XCTAssertTrue(open.waitForExistence(timeout: 5), "Expected Edit File action")
        open.click()

        let modal = app.descendants(matching: .any)[AID.FileEditor.modal]
        XCTAssertTrue(modal.waitForExistence(timeout: 5), "File editor modal did not appear")
        let editor = app.textViews[AID.FileEditor.content]
        XCTAssertTrue(editor.waitForExistence(timeout: 5), "File editor did not appear")
        editor.click()
        editor.typeKey("a", modifierFlags: .command)
        paste("edited inside JayJay\n")

        let save = app.buttons[AID.FileEditor.save]
        XCTAssertTrue(save.waitForExistence(timeout: 5), "Save action did not appear")
        XCTAssertTrue(save.isEnabled, "Modified file should be saveable")
        save.click()
        XCTAssertTrue(editor.waitForNonExistence(timeout: 10), "Editor did not close after saving")

        let fixtureURL = try XCTUnwrap(fixtureURL)
        let saved = try String(contentsOf: fixtureURL.appendingPathComponent("wip1.txt"), encoding: .utf8)
        XCTAssertEqual(saved, "edited inside JayJay\n")
    }
}
