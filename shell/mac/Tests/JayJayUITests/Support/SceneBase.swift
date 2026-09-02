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

    /// The sandboxed runner cannot delete the app defaults, so persisted layout is masked through the argument domain.
    class var startsWithDefaultLayout: Bool {
        true
    }

    override func setUpWithError() throws {
        continueAfterFailure = false
        let root = Self.fixtureRoot
        // Scenes on the same fixture can run in parallel runner clones, so each class gets its own store instead of sharing one per fixture.
        let reviewStorePath = root.appendingPathComponent("\(Self.fixtureName)-\(Self.self)-review-store.json").path
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
        if Self.startsWithDefaultLayout {
            for key in ["jayjay.windowFrame.repo-window", "jayjay.windowFrame.repo-list-window", "jayjay.secondaryPaneWidth", "jayjay.fileColumnWidth"] {
                app.launchArguments += ["-\(key)", ""]
            }
        }
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

    func waitForWindowCount(_ count: Int, in app: XCUIApplication, timeout: TimeInterval = 5) -> Bool {
        wait(for: NSPredicate { _, _ in app.windows.count == count }, object: nil, timeout: timeout)
    }

    func waitForHittable(_ element: XCUIElement, isHittable: Bool, timeout: TimeInterval = 5) -> Bool {
        wait(
            for: NSPredicate(format: "isHittable == %@", NSNumber(value: isHittable)),
            object: element,
            timeout: timeout
        )
    }

    private func wait(for predicate: NSPredicate, object: Any?, timeout: TimeInterval) -> Bool {
        XCTWaiter().wait(
            for: [XCTNSPredicateExpectation(predicate: predicate, object: object)],
            timeout: timeout
        ) == .completed
    }

    func clickDockItem(file: StaticString = #filePath, line: UInt = #line) {
        let tiles = XCUIApplication(bundleIdentifier: "com.apple.dock").dockItems.matching(identifier: "JayJay")
        XCTAssertTrue(
            tiles.firstMatch.waitForExistence(timeout: 3),
            "JayJay was not present in the Dock",
            file: file,
            line: line
        )
        // An installed JayJay keeps its own pinned tile; the app under test is the one the Dock appended last.
        tiles.element(boundBy: tiles.count - 1).click()
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

    /// Toolbar items expose labels, not identifiers.
    static let repositoryTitleLabel = "Switch Repository or Workspace"

    func repositoryTitleButton(in window: XCUIElement) -> XCUIElement {
        window.toolbars.buttons[Self.repositoryTitleLabel].firstMatch
    }

    func openRepositoryTitlePicker(in window: XCUIElement) {
        let titleButton = repositoryTitleButton(in: window)
        XCTAssertTrue(titleButton.waitForExistence(timeout: 10), "Repository title button missing")
        titleButton.click()
    }

    func activateWindow(named identifier: String, in app: XCUIApplication) throws {
        app.menuBars.menuBarItems["Window"].click()
        let items = app.menuItems.matching(identifier: identifier)
        XCTAssertTrue(items.firstMatch.waitForExistence(timeout: 3), "Window menu item \(identifier) missing")
        let item = try XCTUnwrap(
            items.allElementsBoundByIndex.first(where: \.isHittable),
            "Window menu item \(identifier) was not actionable"
        )
        item.click()
    }

    func chooseRepositoryList(in app: XCUIApplication, from window: XCUIElement) {
        let repoList = app.buttons[AID.Picker.row("repo-list")].firstMatch
        openRepositoryTitlePicker(in: window)
        if !repoList.waitForExistence(timeout: 3) {
            // AppKit can swallow the first toolbar click while another window is finishing its close transition.
            keyStroke(.escape)
            openRepositoryTitlePicker(in: window)
        }
        XCTAssertTrue(repoList.waitForExistence(timeout: 3), "Repository List row missing")
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
