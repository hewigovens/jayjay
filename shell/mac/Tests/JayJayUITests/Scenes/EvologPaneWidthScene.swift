import XCTest

final class EvologPaneWidthScene: SceneBase {
    private let initialSnapshotIndex = 2
    private let replacementSnapshotIndex = 4
    private let replacementSnapshotPath = "wip2.txt"
    private let resizeOffset: CGFloat = -30

    override class var fixtureName: String {
        "evolog"
    }

    override class var startsWithDefaultLayout: Bool {
        false
    }

    func testEvologPaneWidthsSurviveSnapshotSwitchAndRelaunch() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10), "DAG never populated")

        rightClickCenter(rows.element(boundBy: 0))
        let showEvolution = app.menuItems["Show evolution…"]
        clickCenter(showEvolution, message: "Show evolution menu item did not appear")

        let entryList = app.descendants(matching: .any)[AID.Evolog.entryList]
        XCTAssertTrue(entryList.waitForExistence(timeout: 10), "Evolog entry list missing")
        let entryListDivider = app.descendants(matching: .any)[AID.Evolog.entryListDivider]
        let entryListInitial = entryList.frame.width
        let entryListTarget = entryListInitial + resizeOffset
        drag(entryListDivider, by: resizeOffset)
        XCTAssertTrue(waitForWidth(entryList, toEqual: entryListTarget), "Narrowing the entry list divider did not resize it")
        let entryListResized = entryList.frame.width

        clickCenter(app.descendants(matching: .any)[AID.Evolog.entry(initialSnapshotIndex)].firstMatch)
        let fileList = app.descendants(matching: .any)[AID.Evolog.fileList]
        XCTAssertTrue(fileList.waitForExistence(timeout: 10), "Evolog file list missing after selecting a version")
        XCTAssertEqual(entryList.frame.width, entryListResized, accuracy: 2, "Selecting a version altered the entry list width")

        let fileListDivider = app.descendants(matching: .any)[AID.Evolog.fileListDivider]
        let fileListInitial = fileList.frame.width
        let fileListTarget = fileListInitial + resizeOffset
        drag(fileListDivider, by: resizeOffset)
        XCTAssertTrue(waitForWidth(fileList, toEqual: fileListTarget), "Narrowing the file list divider did not resize it")
        let fileListResized = fileList.frame.width

        clickCenter(app.descendants(matching: .any)[AID.Evolog.entry(replacementSnapshotIndex)].firstMatch)
        let replacementFile = app.descendants(matching: .any)[AID.Evolog.file(replacementSnapshotPath)]
        XCTAssertTrue(replacementFile.waitForExistence(timeout: 10), "Replacement snapshot never loaded its distinct file")
        XCTAssertEqual(entryList.frame.width, entryListResized, accuracy: 2, "Switching versions altered the entry list width")
        XCTAssertEqual(fileList.frame.width, fileListResized, accuracy: 2, "Switching versions altered the file list width")

        // Both dividers persist through the same shared pane-width preference, so the last drag (the file
        // list's) is what both panes restore to on relaunch.
        app.terminate()
        app.launch()
        rightClickCenter(dagRows(of: app).element(boundBy: 0))
        clickCenter(app.menuItems["Show evolution…"], message: "Show evolution menu item did not appear after relaunch")
        let relaunchedEntryList = app.descendants(matching: .any)[AID.Evolog.entryList]
        XCTAssertTrue(relaunchedEntryList.waitForExistence(timeout: 10), "Evolog entry list missing after relaunch")
        XCTAssertEqual(relaunchedEntryList.frame.width, fileListResized, accuracy: 2, "Entry list width did not restore the shared pane-width preference")
        clickCenter(app.descendants(matching: .any)[AID.Evolog.entry(initialSnapshotIndex)].firstMatch)
        let relaunchedFileList = app.descendants(matching: .any)[AID.Evolog.fileList]
        XCTAssertTrue(relaunchedFileList.waitForExistence(timeout: 10), "Evolog file list missing after relaunch")
        XCTAssertEqual(relaunchedFileList.frame.width, fileListResized, accuracy: 2, "File list width did not restore the shared pane-width preference")
    }

    private func drag(_ divider: XCUIElement, by dx: CGFloat) {
        XCTAssertTrue(divider.waitForExistence(timeout: 5), "Divider missing")
        let grip = divider.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
        grip.press(forDuration: 0.2, thenDragTo: grip.withOffset(CGVector(dx: dx, dy: 0)))
    }

    private func waitForWidth(_ element: XCUIElement, toEqual expected: CGFloat, accuracy: CGFloat = 4) -> Bool {
        let widthSettled = NSPredicate { _, _ in
            abs(element.frame.width - expected) <= accuracy
        }
        return XCTWaiter().wait(
            for: [XCTNSPredicateExpectation(predicate: widthSettled, object: nil)],
            timeout: 5
        ) == .completed
    }
}
