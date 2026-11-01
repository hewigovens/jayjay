import XCTest

final class SettingsNavigationScene: SceneBase {
    func testSidebarSupportsKeyboardNavigation() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: [.command])
        selectSettingsPage("appearance", in: app)
        let window = settingsWindow(in: app)
        XCTAssertTrue(window.waitForExistence(timeout: 5))

        let pages = window.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier BEGINSWITH %@", AID.Settings.page("")))
            .allElementsBoundByIndex
            .compactMap { $0.value as? String }
        XCTAssertEqual(pages.first, "Appearance")
        XCTAssertEqual(pages.last, "About")

        for page in pages.dropFirst() {
            keyStroke(.downArrow)
            XCTAssertTrue(window.wait(for: \.title, toEqual: page, timeout: 5), "Keyboard navigation did not show \(page)")
        }
        XCTAssertTrue(window.descendants(matching: .any).matching(identifier: "Love JayJay?").firstMatch.waitForExistence(timeout: 5))
        keyStroke(.escape)
        XCTAssertTrue(window.waitForNonExistence(timeout: 5))
    }
}
