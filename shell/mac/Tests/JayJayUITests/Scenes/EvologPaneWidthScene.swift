import XCTest

final class EvologPaneWidthScene: SceneBase {
    override class var fixtureName: String {
        "evolog-hide-snapshots"
    }

    override class var startsWithDefaultLayout: Bool {
        false
    }

    // CI runs on a small display; pinning the sidebar at its minimum leaves the three-way evolog split room to drag.
    override class var additionalLaunchArguments: [String] {
        ["-jayjay.sidebarWidth", "<real>240</real>"]
    }

    func testEvologPaneWidthsSurviveVersionSwitchAndRelaunch() throws {
        let app = try XCTUnwrap(app)
        showEvolutionWithSnapshots(in: app)

        let entryList = app.descendants(matching: .any)[AID.Evolog.entryList]
        XCTAssertTrue(entryList.waitForExistence(timeout: 10), "Evolog entry list missing")
        let entryListResized = narrow(entryList, divider: app.descendants(matching: .any)[AID.Evolog.entryListDivider])

        clickCenter(version(2, in: app), message: "Second version row missing")
        let fileList = app.descendants(matching: .any)[AID.Evolog.fileList]
        XCTAssertTrue(fileList.waitForExistence(timeout: 10), "Evolog file list missing after selecting a version")
        XCTAssertEqual(entryList.frame.width, entryListResized, accuracy: 2, "Selecting a version altered the entry list width")
        let fileListResized = narrow(fileList, divider: app.descendants(matching: .any)[AID.Evolog.fileListDivider])

        clickCenter(version(3, in: app), message: "Third version row missing")
        XCTAssertTrue(fileList.waitForExistence(timeout: 10), "Evolog file list missing after switching versions")
        XCTAssertEqual(entryList.frame.width, entryListResized, accuracy: 2, "Switching versions altered the entry list width")
        XCTAssertEqual(fileList.frame.width, fileListResized, accuracy: 2, "Switching versions altered the file list width")

        app.terminate()
        app.launch()
        showEvolutionWithSnapshots(in: app)
        let relaunchedEntryList = app.descendants(matching: .any)[AID.Evolog.entryList]
        XCTAssertTrue(relaunchedEntryList.waitForExistence(timeout: 10), "Evolog entry list missing after relaunch")
        XCTAssertEqual(relaunchedEntryList.frame.width, entryListResized, accuracy: 2, "Entry list width did not persist")
        clickCenter(version(2, in: app), message: "Second version row missing after relaunch")
        let relaunchedFileList = app.descendants(matching: .any)[AID.Evolog.fileList]
        XCTAssertTrue(relaunchedFileList.waitForExistence(timeout: 10), "Evolog file list missing after relaunch")
        XCTAssertEqual(relaunchedFileList.frame.width, entryListResized, accuracy: 2, "File list did not start from the shared pane width")
    }

    private func showEvolutionWithSnapshots(in app: XCUIApplication) {
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10), "DAG never populated")
        rightClickCenter(rows.element(boundBy: 0))
        clickCenter(app.menuItems["Show evolution…"].firstMatch, message: "Show evolution menu item did not appear")
        clickCenter(app.descendants(matching: .any)[AID.Evolog.hideSnapshots].firstMatch, message: "Hide snapshots checkbox missing")
    }

    private func version(_ index: Int, in app: XCUIApplication) -> XCUIElement {
        app.descendants(matching: .any)[AID.Evolog.version(index)].firstMatch
    }

    private func narrow(_ pane: XCUIElement, divider: XCUIElement) -> CGFloat {
        XCTAssertTrue(divider.waitForExistence(timeout: 5), "Pane divider missing")
        let initial = pane.frame.width
        let grip = divider.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
        grip.press(forDuration: 0.2, thenDragTo: grip.withOffset(CGVector(dx: -30, dy: 0)))
        let resized = pane.frame.width
        XCTAssertEqual(resized, initial - 30, accuracy: 4, "Dragging the divider did not resize the pane")
        return resized
    }
}
