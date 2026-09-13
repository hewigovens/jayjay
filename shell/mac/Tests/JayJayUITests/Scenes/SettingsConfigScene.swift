import AppKit
import XCTest

final class SettingsConfigScene: SceneBase {
    override class var launchEnvironment: [String: String] {
        ["JJ_CONFIG": fixtureRoot.appendingPathComponent("missing-jj-config.toml").path]
    }

    func testCopyJjToolConfiguration() throws {
        let app = try XCTUnwrap(app)
        NSPasteboard.general.clearContents()

        app.typeKey(",", modifierFlags: .command)
        selectSettingsTab("CLI", in: app)

        let copyConfig = app.buttons[AID.Settings.copyJJToolConfig]
        XCTAssertTrue(copyConfig.waitForExistence(timeout: 5), "Copy Config button missing")
        copyConfig.click()

        let config = try XCTUnwrap(NSPasteboard.general.string(forType: .string))
        XCTAssertTrue(config.hasPrefix("[merge-tools.jayjay]\n"))
        XCTAssertTrue(config.contains("diff-args ="))
        XCTAssertTrue(config.contains("edit-args ="))
        XCTAssertTrue(config.contains("merge-args ="))
    }

    func testMissingUserConfigShowsEmptyState() throws {
        let app = try XCTUnwrap(app)

        app.typeKey(",", modifierFlags: .command)
        selectSettingsTab("Jujutsu", in: app)

        let missing = app.staticTexts[AID.Settings.jjConfigMissing]
        XCTAssertTrue(missing.waitForExistence(timeout: 5), "Missing config empty state did not appear")
        XCTAssertFalse(app.windows["Jujutsu"].buttons["Open"].exists, "A missing config must not offer Open")
    }
}
