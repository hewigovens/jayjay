import AppKit
import XCTest

final class SettingsConfigScene: SceneBase {
    func testCopyJjToolConfiguration() throws {
        let app = try XCTUnwrap(app)
        NSPasteboard.general.clearContents()

        app.typeKey(",", modifierFlags: .command)
        let cliTab = app.descendants(matching: .any)
            .matching(NSPredicate(format: "label == 'CLI'"))
            .firstMatch
        XCTAssertTrue(cliTab.waitForExistence(timeout: 5), "CLI settings tab missing")
        cliTab.click()

        let copyConfig = app.buttons[AID.Settings.copyJJToolConfig]
        XCTAssertTrue(copyConfig.waitForExistence(timeout: 5), "Copy Config button missing")
        copyConfig.click()

        let config = try XCTUnwrap(NSPasteboard.general.string(forType: .string))
        XCTAssertTrue(config.hasPrefix("[merge-tools.jayjay]\n"))
        XCTAssertTrue(config.contains("diff-args ="))
        XCTAssertTrue(config.contains("edit-args ="))
        XCTAssertTrue(config.contains("merge-args ="))
    }
}
