import XCTest

/// Fixtures are built by `just shell::ui-test-setup` into /tmp/jayjay-test-fixtures; mutating scenes use dedicated generated copies.
class SceneBase: XCTestCase {
    var app: XCUIApplication?
    private(set) var fixtureURL: URL?

    class var fixtureName: String {
        "simple"
    }

    class var launchEnvironment: [String: String] {
        [:]
    }

    class var opensFixtureOnLaunch: Bool {
        true
    }

    class var ignoresWindowRestoration: Bool {
        true
    }

    class var additionalLaunchArguments: [String] {
        []
    }

    class var repositoryStoreFixtureName: String {
        "repositories-empty.json"
    }

    class var fixtureRoot: URL {
        URL(
            fileURLWithPath: ProcessInfo.processInfo.environment["JAYJAY_FIXTURE_ROOT"]
                ?? "/tmp/jayjay-test-fixtures",
            isDirectory: true
        )
    }

    override func setUpWithError() throws {
        continueAfterFailure = false
        let root = Self.fixtureRoot
        let reviewStorePath = root.appendingPathComponent("\(Self.fixtureName)-review-store.json").path
        try? FileManager.default.removeItem(atPath: reviewStorePath)

        let app = XCUIApplication()
        // Ignore restored windows so each test controls its initial app state.
        app.launchArguments = Self.ignoresWindowRestoration
            ? ["-ApplePersistenceIgnoreState", "YES"]
            : []
        if Self.opensFixtureOnLaunch {
            let fixture = root.appendingPathComponent(Self.fixtureName, isDirectory: true)
            fixtureURL = fixture
            app.launchArguments += ["--repo", fixture.path]
        } else if Self.additionalLaunchArguments.isEmpty {
            app.launchArguments += ["-jayjay.lastOpenedRepo", ""]
        }
        app.launchArguments += Self.additionalLaunchArguments
        app.launchEnvironment["JAYJAY_REVIEW_STORE_PATH"] = reviewStorePath
        app.launchEnvironment["JAYJAY_REPOSITORIES_PATH"] = root
            .appendingPathComponent(Self.repositoryStoreFixtureName)
            .path
        for (key, value) in Self.launchEnvironment {
            app.launchEnvironment[key] = value
        }
        app.launch()
        self.app = app
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10), "JayJay window did not appear")
    }

    override func tearDownWithError() throws {
        app?.terminate()
        fixtureURL = nil
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

    func openRepositoryTitleMenu(in window: XCUIElement) {
        let titleMenu = window.toolbars.menuButtons["Switch Repository"].firstMatch
        XCTAssertTrue(titleMenu.waitForExistence(timeout: 10), "Repository title menu missing")
        titleMenu.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 1.2)).click()
    }

    func chooseRepositoryList(in app: XCUIApplication, from window: XCUIElement) {
        let repoList = app.menuItems["Repository List..."]
        openRepositoryTitleMenu(in: window)
        if !repoList.waitForExistence(timeout: 3) {
            // AppKit can swallow the first toolbar-menu click while another window is finishing its close transition.
            keyStroke(.escape)
            openRepositoryTitleMenu(in: window)
        }
        XCTAssertTrue(repoList.waitForExistence(timeout: 3), "Repository List menu item missing")
        repoList.click()
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
