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

    override class var fixtureRoot: URL {
        URL(fileURLWithPath: environment["JAYJAY_SCREENSHOT_ROOT"] ?? "/tmp/jayjay-screenshots", isDirectory: true)
    }

    override class var repositoryStoreFixtureName: String {
        "repositories.json"
    }

    override class var launchEnvironment: [String: String] {
        ["JJ_CONFIG": fixtureRoot.appendingPathComponent("jj-config.toml").path]
    }

    override class var languages: String? {
        environment["JAYJAY_SCREENSHOT_LANGUAGE"].map { "(\($0))" } ?? super.languages
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
            "-jayjay.interfaceFontFamily", "system",
            "-jayjay.interfaceFontSize", "12",
            "-jayjay.treeFileList", "NO",
            "-jayjay.sideBySideDiff", "NO",
            "-jayjay.hasCompletedOnboarding", "YES",
            "-jayjay.windowFrame.repo-window", "{{80, 80}, {1100, 700}}",
            "-jayjay.sidebarWidth", "240",
            "-jayjay.secondaryPaneWidth", "220",
            "-jayjay.fileColumnWidth", "220",
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
        let primaryHeight = NSScreen.screens.first?.frame.height ?? 0
        let screens = NSScreen.screens.map { CGRect(x: $0.frame.minX, y: primaryHeight - $0.frame.maxY, width: $0.frame.width, height: $0.frame.height) }
        let index = screens.firstIndex { $0.contains(CGPoint(x: frame.midX, y: frame.midY)) } ?? 0
        let screen = screens.indices.contains(index) ? screens[index] : CGRect(origin: .zero, size: frame.size)
        let image = XCUIScreen.screens[min(index, XCUIScreen.screens.count - 1)].screenshot().image
        guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
            return XCTFail("Screen capture failed for \(name)")
        }
        let scale = CGFloat(cgImage.width) / screen.width
        let crop = CGRect(
            x: (frame.minX - screen.minX) * scale,
            y: (frame.minY - screen.minY) * scale,
            width: frame.width * scale,
            height: frame.height * scale
        )
        guard let cropped = cgImage.cropping(to: crop.integral) else {
            return XCTFail("Could not crop the capture for \(name)")
        }
        attach(name, image: NSImage(cgImage: cropped, size: frame.size))
    }

    func capture(_ name: String, window: XCUIElement, frame: CGRect? = nil) {
        settle()
        let image = window.screenshot().image
        guard let frame else {
            return attach(name, image: image)
        }
        guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
            return XCTFail("Window capture failed for \(name)")
        }
        let bounds = window.frame
        let scale = CGFloat(cgImage.width) / bounds.width
        let crop = CGRect(x: (frame.minX - bounds.minX) * scale, y: (frame.minY - bounds.minY) * scale, width: frame.width * scale, height: frame.height * scale)
        guard let cropped = cgImage.cropping(to: crop.integral) else {
            return XCTFail("Could not crop the window capture for \(name)")
        }
        attach(name, image: NSImage(cgImage: cropped, size: frame.size))
    }

    private func attach(_ name: String, image: NSImage) {
        let suffix = Self.appearance == "dark" ? "-dark" : ""
        let attachment = XCTAttachment(image: image)
        attachment.name = "\(name)\(suffix)"
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
