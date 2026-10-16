import XCTest

final class CloneRepositoryScene: SceneBase {
    override class var opensFixtureOnLaunch: Bool {
        false
    }

    /// The default location follows the most recent repository, so the scene pins that list instead of reading the runner's own.
    override class var additionalLaunchArguments: [String] {
        let recent = fixtureRoot.appendingPathComponent("formats").path
        return ["-jayjay.lastOpenedRepo", "", "-jayjay.recentRepos", "(\"\(recent)\")"]
    }

    func testCloneReportsTakenFolderThenOpensTheClone() throws {
        let app = try XCTUnwrap(app)
        let open = app.buttons[AID.CloneRepository.open]
        XCTAssertTrue(open.waitForExistence(timeout: 5), "Welcome screen lost Clone Repository")
        open.click()

        let window = app.windows["Clone Repository"]
        XCTAssertTrue(window.waitForExistence(timeout: 5), "Clone window did not open")
        let source = window.textFields[AID.CloneRepository.sourceField]
        let name = window.textFields[AID.CloneRepository.nameField]
        let location = window.textFields[AID.CloneRepository.locationField]
        let clone = window.buttons[AID.CloneRepository.cloneButton]
        XCTAssertTrue(source.waitForExistence(timeout: 5), "URL field missing")
        XCTAssertFalse(clone.isEnabled, "Clone must stay disabled without a URL")

        source.click()
        paste(Self.fixtureRoot.appendingPathComponent("simple").path)
        XCTAssertEqual(name.value as? String, "simple", "Name did not follow the URL")
        XCTAssertEqual(location.value as? String, Self.fixtureRoot.appendingPathComponent("simple").path, "Location did not end in the name")

        // The pinned recent repository puts the default location beside `simple`, so its own name is taken.
        clone.click()
        let error = window.element(AID.CloneRepository.error)
        XCTAssertTrue(error.waitForExistence(timeout: 10), "Taken folder did not report inline")

        let cloneName = "clone-\(UUID().uuidString.prefix(8))"
        name.click()
        keyStroke("a", modifiers: [.command])
        paste(cloneName)
        XCTAssertEqual(location.value as? String, Self.fixtureRoot.appendingPathComponent(cloneName).path, "Location did not follow the name")

        let typedName = "\(cloneName)-typed"
        location.click()
        keyStroke("a", modifiers: [.command])
        paste(Self.fixtureRoot.appendingPathComponent(typedName).path)
        XCTAssertEqual(name.value as? String, typedName, "Name did not follow the location")
        clone.click()

        XCTAssertTrue(app.windows[typedName].waitForExistence(timeout: 30), "Cloned repository did not open")
        XCTAssertTrue(window.waitForNonExistence(timeout: 5), "Clone window stayed open after cloning")

        app.menuBars.menuBarItems["File"].click()
        app.menuItems["Clone Repository..."].click()
        XCTAssertTrue(source.waitForExistence(timeout: 5), "File menu did not reopen the clone window")
        // An empty field can report its prompt as its value, so check for the previous URL rather than for "".
        let reopened = source.value as? String ?? ""
        XCTAssertFalse(reopened.contains(Self.fixtureRoot.path), "A reopened clone window kept the previous URL")
    }
}
