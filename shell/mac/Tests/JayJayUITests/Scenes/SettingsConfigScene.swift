import AppKit
import XCTest

final class SettingsConfigScene: SceneBase {
    func testCopyJjToolConfiguration() throws {
        let app = try XCTUnwrap(app)
        NSPasteboard.general.clearContents()

        app.typeKey(",", modifierFlags: .command)
        let toolsTab = app.descendants(matching: .any)
            .matching(NSPredicate(format: "label == 'Tools'"))
            .firstMatch
        XCTAssertTrue(toolsTab.waitForExistence(timeout: 5), "Tools settings tab missing")
        toolsTab.click()

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
