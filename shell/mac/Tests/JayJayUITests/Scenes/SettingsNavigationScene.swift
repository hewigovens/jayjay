import XCTest

final class SettingsNavigationScene: SceneBase {
    func testSidebarSupportsKeyboardNavigation() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: [.command])
        selectSettingsPage("appearance", in: app)
        let window = settingsWindow(in: app)
        XCTAssertTrue(window.waitForExistence(timeout: 5))

        let pages: [(String, [String])] = [
            ("Appearance", ["Theme"]),
            ("Diff & Files", ["Side-by-side diff"]),
            ("Workflow", ["Confirm before abandoning changes"]),
            ("Integrations", ["Editor"]),
            ("Jujutsu", [AID.Settings.jjConfigPath, AID.Settings.jjConfigMissing]),
            ("Data & Privacy", ["Share anonymous build and OS stats"]),
            ("About", ["Love JayJay?"])
        ]
        for (index, page) in pages.enumerated() {
            if index > 0 {
                keyStroke(.downArrow)
            }
            let shown = page.1.contains { marker in
                window.descendants(matching: .any).matching(identifier: marker).firstMatch.waitForExistence(timeout: 5)
            }
            XCTAssertTrue(shown, "Keyboard navigation did not show \(page.0)")
            let screenshot = XCTAttachment(screenshot: window.screenshot())
            screenshot.name = "Settings - \(page.0)"
            screenshot.lifetime = .keepAlways
            add(screenshot)
        }
        keyStroke(.escape)
        XCTAssertTrue(window.waitForNonExistence(timeout: 5))
    }
}
