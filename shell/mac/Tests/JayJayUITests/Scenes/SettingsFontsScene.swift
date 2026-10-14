import XCTest

final class SettingsFontsScene: SceneBase {
    override class var additionalLaunchArguments: [String] {
        ["-jayjay.fontFamily", "menlo", "-jayjay.fontSize", "14"]
            + ["interfaceFontFamily", "interfaceFontSize", "codeLineSpacing", "codeLigatures"].flatMap { ["-jayjay.\($0)", ""] }
    }

    func testCodeFontControlsUpdateAndReset() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: [.command])
        selectSettingsPage("appearance", in: app)
        let window = settingsWindow(in: app)
        let interfaceSize = window.staticTexts[AID.Settings.interfaceFontSize]
        XCTAssertTrue(interfaceSize.waitForExistence(timeout: 5), "Interface size missing")
        XCTAssertEqual(interfaceSize.value as? String, "14 pt", "Interface size should start from the pre-split size")

        selectSettingsPage("fonts", in: app)
        let codeSize = window.staticTexts[AID.Settings.codeFontSize]
        let lineSpacing = window.staticTexts[AID.Settings.codeLineSpacing]
        XCTAssertTrue(codeSize.waitForExistence(timeout: 5), "Code size missing")
        XCTAssertTrue(window.element(AID.Settings.fontPreview).exists, "Preview missing")
        XCTAssertEqual(codeSize.value as? String, "14 pt")
        XCTAssertFalse(window.element(AID.Settings.codeLigatures).isEnabled, "Menlo has no ligatures")
        attachScreenshot(of: window, named: "Code Font - migrated")

        let steppers = window.steppers
        steppers.element(boundBy: 0).incrementArrows.firstMatch.click()
        steppers.element(boundBy: 1).incrementArrows.firstMatch.click()
        XCTAssertEqual(codeSize.value as? String, "15 pt")
        XCTAssertEqual(lineSpacing.value as? String, "1.1")
        codeSize.scroll(byDeltaX: 0, deltaY: -600)
        attachScreenshot(of: window, named: "Code Font - adjusted")

        window.buttons[AID.Settings.resetCodeFont].click()
        XCTAssertEqual(codeSize.value as? String, "12 pt")
        XCTAssertEqual(lineSpacing.value as? String, "1.0")
        selectSettingsPage("appearance", in: app)
        XCTAssertEqual(interfaceSize.value as? String, "14 pt", "Resetting the code font keeps the interface font")
        attachScreenshot(of: window, named: "Appearance")
    }

    private func attachScreenshot(of window: XCUIElement, named name: String) {
        let screenshot = XCTAttachment(screenshot: window.screenshot())
        screenshot.name = name
        screenshot.lifetime = .keepAlways
        add(screenshot)
    }
}
