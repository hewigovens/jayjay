import XCTest

/// Fixtures are built by `just shell::ui-test-setup` into /tmp/jayjay-test-fixtures; override `fixtureName` to pick one such as `simple`, `simple-formats`, or `conflict`.
class SceneBase: XCTestCase {
    var app: XCUIApplication?

    class var fixtureName: String {
        "simple"
    }

    class var launchEnvironment: [String: String] {
        [:]
    }

    override func setUpWithError() throws {
        continueAfterFailure = false
        let root = ProcessInfo.processInfo.environment["JAYJAY_FIXTURE_ROOT"] ?? "/tmp/jayjay-test-fixtures"
        let reviewStorePath = "\(root)/\(Self.fixtureName)-review-store.json"
        try? FileManager.default.removeItem(atPath: reviewStorePath)
        let app = XCUIApplication()
        // `-<key> <value>` populates NSArgumentDomain; skips onboarding on fresh machines.
        app.launchArguments = ["--repo", "\(root)/\(Self.fixtureName)"]
        app.launchEnvironment["JAYJAY_REVIEW_STORE_PATH"] = reviewStorePath
        for (key, value) in Self.launchEnvironment {
            app.launchEnvironment[key] = value
        }
        app.launch()
        self.app = app
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10), "JayJay window did not appear")
    }

    override func tearDownWithError() throws {
        app?.terminate()
    }

    // MARK: - Query helpers

    func dagRows(of app: XCUIApplication) -> XCUIElementQuery {
        app.descendants(matching: .any).matching(NSPredicate(format: "identifier BEGINSWITH 'dag.row.'"))
    }

    func fileRows(of app: XCUIApplication) -> XCUIElementQuery {
        app.descendants(matching: .any).matching(NSPredicate(format: "identifier BEGINSWITH 'file.row.'"))
    }

    func clickCenter(
        _ element: XCUIElement,
        timeout: TimeInterval = 5,
        message: String = "Element did not appear",
        file: StaticString = #filePath,
        line: UInt = #line
    ) {
        XCTAssertTrue(element.waitForExistence(timeout: timeout), message, file: file, line: line)
        element.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5)).click()
    }

    func rightClickCenter(
        _ element: XCUIElement,
        timeout: TimeInterval = 5,
        message: String = "Element did not appear",
        file: StaticString = #filePath,
        line: UInt = #line
    ) {
        XCTAssertTrue(element.waitForExistence(timeout: timeout), message, file: file, line: line)
        element.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5)).rightClick()
    }

    // MARK: - Key input

    /// Types via the pasteboard: XCUIElement.typeText is flaky against custom key handling.
    @nonobjc
    func paste(_ text: String) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(text, forType: .string)
        keyStroke("v", modifiers: [.command])
    }

    func keyStroke(_ key: XCUIKeyboardKey, modifiers: XCUIElement.KeyModifierFlags = []) {
        app?.typeKey(key, modifierFlags: modifiers)
    }

    @nonobjc
    func keyStroke(_ key: String, modifiers: XCUIElement.KeyModifierFlags = []) {
        app?.typeKey(key, modifierFlags: modifiers)
    }
}
