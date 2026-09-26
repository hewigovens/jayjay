import AppKit
import XCTest

/// Launch configuration and capture helpers shared by the public screenshot scenes; see agents/release.md.
/// Skipped unless JAYJAY_SCREENSHOT_APPEARANCE (light or dark) reaches the runner, so CI never runs them.
class ScreenshotScene: SceneBase {
    static let environment = ProcessInfo.processInfo.environment
    private static let appearance = environment["JAYJAY_SCREENSHOT_APPEARANCE"]
    private(set) var mainFrame = CGRect.zero

    override class var fixtureName: String {
        "flightdeck"
    }

    override class var repositoryStoreFixtureName: String {
        "repositories.json"
    }

    override class var launchEnvironment: [String: String] {
        ["JJ_CONFIG": fixtureRoot.appendingPathComponent("jj-config.toml").path]
    }

    override class var startsWithDefaultLayout: Bool {
        false
    }

    override class var additionalLaunchArguments: [String] {
        [
            "-jayjay.appearanceMode", appearance ?? "light",
            "-jayjay.tintWindowWithWallpaper", "NO",
            "-jayjay.fontFamily", "system",
            "-jayjay.fontSize", "12",
            "-jayjay.treeFileList", "NO",
            "-jayjay.sideBySideDiff", "NO",
            "-jayjay.hasCompletedOnboarding", "YES",
            "-jayjay.windowFrame.repo-window", "{{80, 80}, {1600, 960}}",
            "-jayjay.sidebarWidth", "",
            "-jayjay.secondaryPaneWidth", "",
            "-jayjay.fileColumnWidth", "",
            "-jayjay.sidebarHidden", "NO",
            "-commandPalette.frameOrigin", "{0, 0}"
        ]
    }

    override func setUpWithError() throws {
        try XCTSkipIf(Self.appearance == nil, "Set JAYJAY_SCREENSHOT_APPEARANCE to capture release screenshots")
        try super.setUpWithError()
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 15), "DAG never populated")
        mainFrame = app.windows.firstMatch.frame
        settle()
    }

    /// Rows expose their selection revision, not their text, so the fixture writes id-subject pairs to rows.tsv.
    static func fixtureRowId(for subject: String) -> String {
        let rows = (try? String(contentsOf: fixtureRoot.appendingPathComponent("rows.tsv"), encoding: .utf8)) ?? ""
        return rows.split(separator: "\n")
            .map { $0.split(separator: "\t", maxSplits: 1).map(String.init) }
            .first { $0.count == 2 && $0[1].contains(subject) }?[0] ?? subject
    }

    /// Lets highlighting, avatars, and animations finish before the capture.
    func settle() {
        _ = XCTWaiter().wait(for: [XCTestExpectation(description: "settle")], timeout: 1.5)
    }

    /// A screen capture cropped to the window, so open menus and popovers are included and the pointer is not.
    func capture(_ name: String, frame: CGRect) {
        settle()
        let screenshot = XCUIScreen.main.screenshot()
        let image = screenshot.image
        guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
            return XCTFail("Screen capture failed for \(name)")
        }
        let scale = CGFloat(cgImage.width) / (NSScreen.screens.first?.frame.width ?? image.size.width)
        let crop = CGRect(x: frame.minX * scale, y: frame.minY * scale, width: frame.width * scale, height: frame.height * scale)
        guard let cropped = cgImage.cropping(to: crop.integral) else {
            return XCTFail("Could not crop the capture for \(name)")
        }
        let suffix = Self.appearance == "dark" ? "-dark" : ""
        let attachment = XCTAttachment(image: NSImage(cgImage: cropped, size: frame.size))
        attachment.name = "\(name)\(suffix)"
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
