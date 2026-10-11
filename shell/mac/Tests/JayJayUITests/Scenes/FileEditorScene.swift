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

        let preparing = app.descendants(matching: .any)[AID.FileEditor.preparing]
        XCTAssertTrue(preparing.waitForNonExistence(timeout: 60), "File editor preparation did not finish")
        let modal = app.descendants(matching: .any)[AID.FileEditor.modal]
        XCTAssertTrue(modal.waitForExistence(timeout: 10), "File editor modal did not appear")
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

    func testDiffLineOpensTheEditorWithTheCaretOnThatLine() throws {
        let app = try XCTUnwrap(app)
        let file = fileRows(of: app)
            .matching(NSPredicate(format: "identifier == %@", AID.FileList.row("lines.txt")))
            .firstMatch
        XCTAssertTrue(file.waitForExistence(timeout: 10), "Expected lines.txt row")
        file.click()

        let gutter = app.textViews[AID.Diff.gutter]
        XCTAssertTrue(gutter.waitForExistence(timeout: 5), "Diff gutter did not appear")
        gutter.coordinate(withNormalizedOffset: .zero).withOffset(CGVector(dx: 12, dy: 40)).rightClick()
        let item = app.menuItems.matching(NSPredicate(format: "title BEGINSWITH %@", "Edit File at Line ")).firstMatch
        XCTAssertTrue(item.waitForExistence(timeout: 5), "Expected an Edit File at Line item")
        let line = try XCTUnwrap(Int(item.title.dropFirst("Edit File at Line ".count)))
        XCTAssertGreaterThan(line, 1, "Expected a row below the first line")
        item.click()

        let editor = app.textViews[AID.FileEditor.content]
        XCTAssertTrue(editor.waitForExistence(timeout: 60), "File editor did not appear")
        paste(">")
        let save = app.buttons[AID.FileEditor.save]
        XCTAssertTrue(save.waitForExistence(timeout: 5), "Save action did not appear")
        save.click()
        XCTAssertTrue(editor.waitForNonExistence(timeout: 10), "Editor did not close after saving")

        let fixtureURL = try XCTUnwrap(fixtureURL)
        let saved = try String(contentsOf: fixtureURL.appendingPathComponent("lines.txt"), encoding: .utf8)
        let lines = ["one", "two", "three", "four", "five", "SIX"]
        XCTAssertEqual(saved.split(separator: "\n").map(String.init), lines.enumerated().map { $0.offset == line - 1 ? ">" + $0.element : $0.element })
    }
}
