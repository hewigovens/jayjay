import XCTest

final class FileSelectionScrollScene: SceneBase {
    override class var fixtureName: String {
        "file-selection-scroll"
    }

    override class var additionalLaunchArguments: [String] {
        ["-jayjay.treeFileList", "NO"]
    }

    func testFlatListRevealsKeyboardSelection() throws {
        try checkKeyboardSelection(tree: false)
    }

    func testTreeListRevealsKeyboardSelection() throws {
        try launchTreeMode()
        try checkKeyboardSelection(tree: true)
    }

    func testRefreshKeepsTheSelectedFileVisible() throws {
        let app = try XCTUnwrap(app)
        let first = fileRows(of: app).matching(identifier: AID.FileList.row("assets/logo.bin")).firstMatch
        let last = fileRows(of: app).matching(identifier: AID.FileList.row("src/modules/module-08.swift")).firstMatch
        XCTAssertTrue(first.waitForExistence(timeout: 10))
        app.buttons[AID.Detail.descriptionExpansion].click()
        let commit = app.descendants(matching: .any)[AID.Detail.commitId].firstMatch
        XCTAssertTrue(commit.waitForExistence(timeout: 5))
        let originalCommit = try XCTUnwrap(commit.value as? String)
        first.click()
        for _ in 0 ..< 50 {
            keyStroke("j")
        }
        XCTAssertTrue(last.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(last, isHittable: true))
        XCTAssertFalse(first.isHittable)

        let path = try XCTUnwrap(fixtureURL).appendingPathComponent("src/modules/module-08.swift")
        let original = try String(contentsOf: path, encoding: .utf8)
        app.buttons["Edit File"].click()
        let editor = app.textViews[AID.FileEditor.content]
        XCTAssertTrue(editor.waitForExistence(timeout: 10))
        editor.click()
        keyStroke("a", modifiers: .command)
        paste(original + "\n// Refresh visibility regression\n")
        app.buttons[AID.FileEditor.save].click()
        XCTAssertTrue(editor.waitForNonExistence(timeout: 10))
        keyStroke("r", modifiers: .command)
        let refreshed = NSPredicate(format: "value != %@", originalCommit)
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: refreshed, object: commit)], timeout: 10),
            .completed
        )
        XCTAssertTrue(last.isSelected)
        XCTAssertTrue(waitForHittable(last, isHittable: true))
    }

    func testTreeNavigationFollowsVisibleRowsAndSkipsCollapsedFolders() throws {
        let app = try XCTUnwrap(app)
        try launchTreeMode()
        let first = fileRows(of: app).matching(identifier: AID.FileList.row("assets/logo.bin")).firstMatch
        let rootRow = fileRows(of: app).matching(identifier: AID.FileList.row("a-root.txt")).firstMatch
        let config = fileRows(of: app).matching(identifier: AID.FileList.row("config/environments/development.json")).firstMatch
        let guide = fileRows(of: app).matching(identifier: AID.FileList.row("docs/guides/guide-01.md")).firstMatch
        XCTAssertTrue(first.waitForExistence(timeout: 10))
        first.click()
        keyStroke("k")
        XCTAssertTrue(first.wait(for: \.isSelected, toEqual: true, timeout: 5))
        keyStroke("j")
        XCTAssertTrue(config.wait(for: \.isSelected, toEqual: true, timeout: 5))
        app.buttons["config/environments"].click()
        first.click()
        keyStroke("j")
        XCTAssertTrue(guide.wait(for: \.isSelected, toEqual: true, timeout: 5))
        keyStroke(.space)
        XCTAssertTrue(first.wait(for: \.isSelected, toEqual: true, timeout: 5))
        for _ in 0 ..< 50 {
            keyStroke("j")
        }
        XCTAssertTrue(rootRow.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(rootRow, isHittable: true))
        keyStroke("k")
        let lastFolderFile = fileRows(of: app).matching(identifier: AID.FileList.row("src/modules/module-08.swift")).firstMatch
        XCTAssertTrue(lastFolderFile.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(lastFolderFile, isHittable: true))
    }

    func testSpaceRevealsNextUnreviewedFileInCollapsedFolder() throws {
        try launchTreeMode()
        let app = try XCTUnwrap(app)
        let development = fileRows(of: app).matching(identifier: AID.FileList.row("config/environments/development.json")).firstMatch
        let staging = fileRows(of: app).matching(identifier: AID.FileList.row("config/environments/staging.json")).firstMatch
        let feature = fileRows(of: app).matching(identifier: AID.FileList.row("src/features/area-01/component.swift")).firstMatch
        XCTAssertTrue(development.waitForExistence(timeout: 10))
        app.buttons[AID.FileList.filterToggle].click()
        XCTAssertTrue(app.descendants(matching: .any)[AID.FileList.filterField].waitForExistence(timeout: 5))
        paste("ent")
        keyStroke(.return)
        XCTAssertTrue(development.wait(for: \.isSelected, toEqual: true, timeout: 5))
        app.buttons["src/features"].click()
        XCTAssertTrue(feature.waitForNonExistence(timeout: 5))

        keyStroke(.space)
        XCTAssertTrue(staging.wait(for: \.isSelected, toEqual: true, timeout: 5))
        keyStroke(.space)
        XCTAssertTrue(feature.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(feature, isHittable: true))
    }

    private func launchTreeMode() throws {
        let app = try XCTUnwrap(app)
        app.terminate()
        let argument = try XCTUnwrap(app.launchArguments.firstIndex(of: "-jayjay.treeFileList"))
        app.launchArguments[argument + 1] = "YES"
        app.launch()
    }

    private func checkKeyboardSelection(tree: Bool) throws {
        let app = try XCTUnwrap(app)
        let first = fileRows(of: app).matching(identifier: AID.FileList.row(tree ? "assets/logo.bin" : "a-root.txt")).firstMatch
        let last = fileRows(of: app).matching(identifier: AID.FileList.row(tree ? "a-root.txt" : "src/modules/module-08.swift")).firstMatch
        XCTAssertTrue(first.waitForExistence(timeout: 10))
        XCTAssertTrue(first.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(first, isHittable: true))
        XCTAssertFalse(last.isHittable)
        first.click()

        for _ in 0 ..< 50 {
            keyStroke("j")
        }
        XCTAssertTrue(last.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(last, isHittable: true))
        XCTAssertFalse(first.isHittable)

        for _ in 0 ..< 50 {
            keyStroke("k")
        }
        XCTAssertTrue(first.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(first, isHittable: true))

        for _ in 0 ..< 50 {
            keyStroke(.downArrow)
        }
        XCTAssertTrue(last.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(last, isHittable: true))
        keyStroke(.upArrow)
        let previous = fileRows(of: app).matching(identifier: AID.FileList.row(tree ? "src/modules/module-08.swift" : "src/modules/module-07.swift")).firstMatch
        XCTAssertTrue(previous.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(previous, isHittable: true))
        XCTAssertFalse(first.isHittable)

        keyStroke(.space)
        XCTAssertTrue(first.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertTrue(waitForHittable(first, isHittable: true))
        XCTAssertEqual(app.buttons[AID.FileList.review(tree ? "src/modules/module-08.swift" : "src/modules/module-07.swift")].label, "Reviewed")
    }
}
