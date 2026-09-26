import XCTest

/// Captures the public screenshots in docs/imgs from the scripts/screenshot-fixture.sh repository; see agents/release.md.
final class ReleaseScreenshots: ScreenshotScene {
    // MARK: - Main window

    func testHome() throws {
        let app = try XCTUnwrap(app)
        selectChange("include transfer time", in: app)
        selectFile("src/routes.rs", in: app)
        clickCenter(app.buttons["Unified"], message: "Diff layout toggle missing")
        capture("home", frame: mainFrame)
    }

    func testReview() throws {
        let app = try XCTUnwrap(app)
        selectFile("src/stations.rs", in: app)
        let gutter = app.textViews[AID.Diff.gutter]
        XCTAssertTrue(gutter.waitForExistence(timeout: 5), "Diff gutter did not appear")
        let addNote = app.menuItems["Add Review Note"]
        for _ in 0..<3 where !addNote.exists {
            gutter.coordinate(withNormalizedOffset: .zero).withOffset(CGVector(dx: 12, dy: 12)).rightClick()
            if !addNote.waitForExistence(timeout: 3) {
                keyStroke(.escape)
            }
        }
        clickCenter(addNote, message: "Add Review Note missing")
        XCTAssertTrue(app.textViews[AID.ReviewNote.body].waitForExistence(timeout: 5), "Note editor missing")
        paste("Should plan_route call this before filtering candidates?")
        clickCenter(app.buttons["Add Note"], message: "Add Note missing")
        settle()
        gutter.coordinate(withNormalizedOffset: .zero).withOffset(CGVector(dx: 12, dy: 12)).click()
        clickCenter(app.buttons[AID.FileList.review("README.md")], message: "README review control missing")
        selectFile("src/stations.rs", in: app)
        capture("review", frame: mainFrame)
    }

    func testSideBySide() throws {
        let app = try XCTUnwrap(app)
        selectChange("include transfer time", in: app)
        selectFile("src/cache.rs", in: app)
        capture("side-by-side", frame: mainFrame)
    }

    func testDivergentDiff() throws {
        let app = try XCTUnwrap(app)
        selectLowChange("comfort (weighted)", in: app)
        capture("divergent-diff", frame: mainFrame)
    }

    func testInterdiff() throws {
        let app = try XCTUnwrap(app)
        selectChange("prefer the fastest", in: app)
        XCUIElement.perform(withKeyModifiers: .command) {
            self.row("include transfer time", in: app).click()
        }
        XCTAssertTrue(app.descendants(matching: .any)[AID.Compare.banner].waitForExistence(timeout: 5), "Compare banner missing")
        selectFile("src/routes.rs", in: app)
        capture("interdiff", frame: mainFrame)
    }

    func testDiffEdit() throws {
        let app = try XCTUnwrap(app)
        selectChange("include transfer time", in: app)
        clickCenter(app.buttons[AID.DiffEdit.open], message: "Edit Diff missing")
        XCTAssertTrue(app.buttons[AID.DiffEdit.cancel].waitForExistence(timeout: 5), "Diff Edit did not open")
        capture("diff-edit", frame: mainFrame)
    }

    func testConflictResolution() throws {
        let app = try XCTUnwrap(app)
        selectChange("merge rail and ferry", in: app)
        selectFile("src/timetable.rs", in: app)
        capture("conflict-resolution", frame: mainFrame)
    }

    func testConflictResolver() throws {
        let app = try XCTUnwrap(app)
        selectChange("merge rail and ferry", in: app)
        clickCenter(app.buttons[AID.Conflict.resolveInJayJay("src/timetable.rs")], timeout: 10, message: "Edit in JayJay missing")
        XCTAssertTrue(app.descendants(matching: .any)[AID.Conflict.editorPreparing].waitForNonExistence(timeout: 60))
        XCTAssertTrue(app.staticTexts[AID.Conflict.editorModal].waitForExistence(timeout: 10), "Conflict editor missing")
        capture("conflict-resolver", frame: mainFrame)
    }

    func testEvolog() throws {
        let app = try XCTUnwrap(app)
        rightClickCenter(row("include transfer time", in: app))
        clickCenter(app.menuItems["Show evolution…"].firstMatch, message: "Show evolution missing")
        let toggle = app.descendants(matching: .any)[AID.Evolog.hideSnapshots].firstMatch
        XCTAssertTrue(toggle.waitForExistence(timeout: 5), "Evolution did not open")
        let versions = (0..<8).map { app.descendants(matching: .any)[AID.Evolog.version($0)].firstMatch }
        let oldest = try XCTUnwrap(versions.last { $0.exists }, "No evolution versions")
        oldest.click()
        capture("evolog", frame: mainFrame)
    }

    func testStackedPr() throws {
        let app = try XCTUnwrap(app)
        rightClickCenter(row("include transfer time", in: app))
        clickCenter(app.menuItems["Create / Update Stacked PRs…"], message: "Stacked PRs menu item missing")
        XCTAssertTrue(app.staticTexts["Stacked Pull Requests"].waitForExistence(timeout: 10), "Stacked PR panel missing")
        capture("stacked-pr", frame: mainFrame)
    }

    func testStackedPrMenu() throws {
        let app = try XCTUnwrap(app)
        selectChange("include transfer time", in: app)
        rightClickCenter(row("include transfer time", in: app))
        let item = app.menuItems["Create / Update Stacked PRs…"]
        XCTAssertTrue(item.waitForExistence(timeout: 3), "Stacked PRs menu item missing")
        item.hover()
        capture("stacked-pr-menu", frame: mainFrame)
    }

    // MARK: - Windows and panels

    func testBookmarkManager() throws {
        let app = try XCTUnwrap(app)
        keyStroke("b", modifiers: [.command, .shift])
        XCTAssertTrue(app.staticTexts["Bookmark Manager"].waitForExistence(timeout: 5), "Bookmark Manager missing")
        capture("bookmark-manager", frame: mainFrame)
    }

    func testOperationLog() throws {
        let app = try XCTUnwrap(app)
        keyStroke("u", modifiers: [.command, .shift])
        XCTAssertTrue(app.staticTexts["Operation Log"].waitForExistence(timeout: 5), "Operation Log missing")
        capture("undo-operation-log", frame: mainFrame)
    }

    func testCommandPalette() throws {
        let app = try XCTUnwrap(app)
        selectLowChange("try weighted routing", in: app)
        let field = openPalette(app)
        paste("bookmark")
        capture("command-palette", frame: palette(app))
        field.click()
        keyStroke("a", modifiers: [.command])
        paste("jj ")
        capture("command-palette-jj-raw", frame: palette(app))
    }

    func testWorkspaces() throws {
        let app = try XCTUnwrap(app)
        openRepositoryTitlePicker(in: mainWindow(app))
        let picker = app.dialogs.containing(.any, identifier: AID.Picker.row("repo-list")).firstMatch
        XCTAssertTrue(picker.waitForExistence(timeout: 5), "Workspace picker missing")
        // Moving off the title button keeps its tooltip out of the capture.
        row("add journey planning", in: app).hover()
        capture("workspaces", frame: picker.frame)
    }

    func testSettings() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: [.command])
        let pages = [
            ("appearance", "appearance"), ("diff", "diff"), ("workflow", "workflow"), ("integrations", "integrations"),
            ("jujutsu", "jj"), ("dataPrivacy", "data-privacy"), ("about", "about")
        ]
        for (page, name) in pages {
            selectSettingsPage(page, in: app)
            settle()
            capture("settings-\(name)", frame: settingsWindow(in: app).frame)
        }
    }

    // MARK: - Helpers

    private func mainWindow(_ app: XCUIApplication) -> XCUIElement {
        app.windows.firstMatch
    }

    private func palette(_ app: XCUIApplication) -> CGRect {
        let field = app.textFields[AID.Palette.textField]
        for query in [app.dialogs, app.windows] {
            let panel = query.containing(.textField, identifier: AID.Palette.textField).firstMatch
            if panel.exists {
                return panel.frame
            }
        }
        return CGRect(x: field.frame.minX - 40, y: field.frame.minY - 12, width: 520, height: 360)
    }

    private func row(_ subject: String, in app: XCUIApplication) -> XCUIElement {
        app.descendants(matching: .any)[AID.DAG.row(Self.fixtureRowId(for: subject))].firstMatch
    }

    private func selectChange(_ subject: String, in app: XCUIApplication) {
        clickCenter(row(subject, in: app), timeout: 10, message: "Change \(subject) missing")
        settle()
    }

    /// Leaving the working copy hides the commit box, so rows near the bottom of the graph fit on screen.
    private func selectLowChange(_ subject: String, in app: XCUIApplication) {
        selectChange("merge rail and ferry", in: app)
        selectChange(subject, in: app)
    }

    private func selectFile(_ path: String, in app: XCUIApplication) {
        let file = app.descendants(matching: .any).matching(NSPredicate(format: "identifier == %@", AID.FileList.row(path))).firstMatch
        clickCenter(file, timeout: 10, message: "\(path) row missing")
        XCTAssertTrue(app.descendants(matching: .any)[AID.Diff.section].waitForExistence(timeout: 5), "Diff did not load")
        settle()
    }

    private func openPalette(_ app: XCUIApplication) -> XCUIElement {
        keyStroke("p", modifiers: [.command, .shift])
        let field = app.textFields[AID.Palette.textField]
        XCTAssertTrue(field.waitForExistence(timeout: 5), "Command palette missing")
        return field
    }
}
