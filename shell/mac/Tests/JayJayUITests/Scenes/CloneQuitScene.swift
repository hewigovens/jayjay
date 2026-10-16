import XCTest

final class CloneQuitScene: SceneBase {
    override class var opensFixtureOnLaunch: Bool {
        false
    }

    override class var additionalLaunchArguments: [String] {
        let recent = fixtureRoot.appendingPathComponent("formats").path
        return ["-jayjay.lastOpenedRepo", "", "-jayjay.recentRepos", "(\"\(recent)\")"]
    }

    /// The fixture's Git sleeps instead of fetching, so the clone is still running when JayJay quits.
    override class var launchEnvironment: [String: String] {
        ["JJ_CONFIG": fixtureRoot.appendingPathComponent("clone-hang-jj-config.toml").path]
    }

    func testQuittingStopsARunningCloneAndRemovesItsFolder() throws {
        let app = try XCTUnwrap(app)
        app.buttons[AID.CloneRepository.open].click()
        let window = app.windows["Clone Repository"]
        let source = window.textFields[AID.CloneRepository.sourceField]
        XCTAssertTrue(source.waitForExistence(timeout: 5), "Clone window did not open")

        let name = "hang-\(UUID().uuidString.prefix(8))"
        source.click()
        paste("https://example.invalid/\(name).git")
        window.buttons[AID.CloneRepository.cloneButton].click()
        let destination = Self.fixtureRoot.appendingPathComponent(name).path
        XCTAssertTrue(app.staticTexts["Cloning \(name)…"].waitForExistence(timeout: 5), "Clone did not start")
        let deadline = Date().addingTimeInterval(10)
        while !FileManager.default.fileExists(atPath: destination), Date() < deadline {
            RunLoop.current.run(until: Date().addingTimeInterval(0.1))
        }
        XCTAssertTrue(FileManager.default.fileExists(atPath: destination), "jj never created the destination")

        keyStroke("q", modifiers: [.command])
        XCTAssertTrue(app.wait(for: .notRunning, timeout: 10), "JayJay did not quit")
        XCTAssertFalse(FileManager.default.fileExists(atPath: destination), "Quitting left the partial clone behind")
    }
}
