import XCTest

final class SettingsResponsivenessScene: SceneBase {
    override class var additionalLaunchArguments: [String] {
        ["-jayjay.skipAbandonConfirmation", "NO"]
    }

    override class var launchEnvironment: [String: String] {
        let path = ProcessInfo.processInfo.environment["PATH"] ?? "/usr/bin:/bin"
        return ["PATH": "\(fixtureRoot.appendingPathComponent("settings-tools").path):\(path)"]
    }

    func testDiffTogglesDoNotRunCliProbesAndTabsReuseDetection() throws {
        let app = try XCTUnwrap(app)
        let binary = Self.fixtureRoot.appendingPathComponent("settings-tools/glab")
        let calls = binary.appendingPathExtension("calls")
        keyStroke(",", modifiers: [.command])
        selectTab("CLI", in: app)
        let detectedPath = app.windows["CLI"].staticTexts[binary.path]
        XCTAssertTrue(detectedPath.waitForExistence(timeout: 10), "background detection never finished")
        XCTAssertEqual(try String(contentsOf: calls, encoding: .utf8), "probe\n")

        selectTab("Diff", in: app)
        let toggle = app.windows["Diff"].switches[AID.Settings.skipAbandonConfirmation]
        XCTAssertTrue(toggle.waitForExistence(timeout: 5))
        toggle.click()
        toggle.click()
        XCTAssertEqual(try String(contentsOf: calls, encoding: .utf8), "probe\n", "Diff toggles must not probe CLI tools")
        selectTab("CLI", in: app)
        XCTAssertTrue(detectedPath.waitForExistence(timeout: 5))
        XCTAssertEqual(try String(contentsOf: calls, encoding: .utf8), "probe\n", "tab changes must reuse the loaded detection")
    }

    private func selectTab(_ name: String, in app: XCUIApplication) {
        let tab = app.descendants(matching: .any)
            .matching(NSPredicate(format: "label == %@", name))
            .firstMatch
        XCTAssertTrue(tab.waitForExistence(timeout: 5), "Settings tab \(name) missing")
        tab.click()
    }
}
