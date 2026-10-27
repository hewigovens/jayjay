import XCTest

final class OpenRemoteRepositoryScene: SceneBase {
    override class var additionalLaunchArguments: [String] {
        ["-jayjay.appearanceMode", "light", "-jayjay.interfaceFontSize", "12"]
    }

    func testRemotePickerShowsFoldersOnlyAfterConnecting() throws {
        let app = try XCTUnwrap(app)
        app.menuBars.menuBarItems["File"].click()
        app.menuItems["Open Remote Repository…"].click()
        let window = app.windows["Open Remote Repository"]
        XCTAssertTrue(window.waitForExistence(timeout: 5))
        let machine = window.textFields[AID.OpenRemoteRepository.machine]
        let connect = window.buttons[AID.OpenRemoteRepository.connect]
        XCTAssertTrue(machine.exists)
        XCTAssertFalse(window.textFields[AID.OpenRemoteRepository.path].exists)
        XCTAssertFalse(connect.isEnabled)
        XCTAssertFalse(window.buttons[AID.OpenRemoteRepository.open].isEnabled)

        machine.click()
        paste("alice@development-machine-with-a-long-name.example.com")
        XCTAssertTrue(connect.isEnabled)
        XCTAssertTrue(window.buttons[AID.OpenRemoteRepository.open].isHittable)
        let screenshot = XCTAttachment(screenshot: window.screenshot())
        screenshot.name = "Remote repository picker"
        screenshot.lifetime = .keepAlways
        add(screenshot)

        keyStroke(.escape)
        XCTAssertTrue(window.waitForNonExistence(timeout: 5))
    }
}
