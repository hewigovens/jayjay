import XCTest

final class SettingsAgentSkillsScene: SceneBase {
    override class var launchEnvironment: [String: String] {
        ["JAYJAY_SKILLS_HOME": fixtureRoot.appendingPathComponent("agent-skills-home").path]
    }

    func testInstallsAndRemovesSkillFromSettings() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: [.command])
        selectSettingsPage("integrations", in: app)
        let window = settingsWindow(in: app)
        let toggle = window.switches[AID.Settings.agentSkill]
        XCTAssertTrue(toggle.waitForExistence(timeout: 5))
        XCTAssertTrue(toggle.wait(for: \.isEnabled, toEqual: true, timeout: 5))
        XCTAssertEqual(toggle.value as? Int, 0)
        toggle.click()
        assertSkillValue(1, toggle: toggle)
        let screenshot = XCTAttachment(screenshot: window.screenshot())
        screenshot.name = "Coding Agents - installed"
        screenshot.lifetime = .keepAlways
        add(screenshot)

        selectSettingsPage("workflow", in: app)
        selectSettingsPage("integrations", in: app)
        XCTAssertTrue(toggle.wait(for: \.isEnabled, toEqual: true, timeout: 5))
        XCTAssertEqual(toggle.value as? Int, 1)
        toggle.click()
        assertSkillValue(0, toggle: toggle)
    }

    private func assertSkillValue(_ value: Int, toggle: XCUIElement) {
        let predicate = NSPredicate { _, _ in
            toggle.isEnabled && toggle.value as? Int == value
        }
        XCTAssertEqual(XCTWaiter.wait(for: [XCTNSPredicateExpectation(predicate: predicate, object: nil)], timeout: 5), .completed)
    }
}
