import XCTest

final class SettingsResponsivenessScene: SceneBase {
    override class var additionalLaunchArguments: [String] {
        ["-jayjay.skipAbandonConfirmation", "NO"]
    }

    override class var launchEnvironment: [String: String] {
        let path = ProcessInfo.processInfo.environment["PATH"] ?? "/usr/bin:/bin"
        return ["PATH": "\(fixtureRoot.appendingPathComponent("settings-tools").path):\(path)"]
    }

    func testWorkflowTogglesDoNotRunCliProbesAndPagesReuseDetection() throws {
        let app = try XCTUnwrap(app)
        let binary = Self.fixtureRoot.appendingPathComponent("settings-tools/glab")
        let calls = binary.appendingPathExtension("calls")
        keyStroke(",", modifiers: [.command])
        selectSettingsPage("integrations", in: app)
        let detectedPath = settingsWindow(in: app).staticTexts[binary.path]
        XCTAssertTrue(detectedPath.waitForExistence(timeout: 10), "background detection never finished")
        XCTAssertEqual(try String(contentsOf: calls, encoding: .utf8), "probe\n")

        selectSettingsPage("workflow", in: app)
        let toggle = settingsWindow(in: app).switches[AID.Settings.confirmAbandon]
        XCTAssertTrue(toggle.waitForExistence(timeout: 5))
        XCTAssertEqual(toggle.value as? Int, 1, "Confirm must be enabled when skipAbandonConfirmation is false")
        toggle.click()
        XCTAssertEqual(toggle.value as? Int, 0)
        selectSettingsPage("diff", in: app)
        selectSettingsPage("workflow", in: app)
        XCTAssertEqual(toggle.value as? Int, 0, "Confirmation preference must survive navigation")
        toggle.click()
        XCTAssertEqual(toggle.value as? Int, 1)
        XCTAssertEqual(try String(contentsOf: calls, encoding: .utf8), "probe\n", "Workflow toggles must not probe CLI tools")
        selectSettingsPage("integrations", in: app)
        XCTAssertTrue(detectedPath.waitForExistence(timeout: 5))
        XCTAssertEqual(try String(contentsOf: calls, encoding: .utf8), "probe\n", "page changes must reuse the loaded detection")
    }
}
