import XCTest

final class EvologHideSnapshotsScene: SceneBase {
    override class var fixtureName: String {
        "evolog-hide-snapshots"
    }

    func testToggleHideSnapshotsAndExpandCollapsedRun() throws {
        let app = try XCTUnwrap(app)
        let toggle = openEvolution(in: app)

        let collapsed = collapsedSnapshotRun(in: app)
        XCTAssertTrue(collapsed.waitForExistence(timeout: 5), "Collapsed snapshot run did not appear")

        clickCenter(toggle, message: "Hide snapshots checkbox was not hittable")
        XCTAssertTrue(
            collapsed.waitForNonExistence(timeout: 5),
            "Turning hide off should show every snapshot row"
        )

        clickCenter(toggle, message: "Hide snapshots checkbox was not hittable")
        XCTAssertTrue(
            collapsedSnapshotRun(in: app).waitForExistence(timeout: 5),
            "Turning hide on should collapse consecutive snapshots"
        )

        let run = collapsedSnapshotRun(in: app)
        clickCenter(run, message: "Collapsed snapshot run was not hittable")
        XCTAssertTrue(
            run.waitForNonExistence(timeout: 5),
            "Clicking a collapsed run should expand it"
        )
    }

    func testCommandAndShiftClickDiffSelectedVersions() throws {
        let app = try XCTUnwrap(app)
        let toggle = openEvolution(in: app)
        clickCenter(toggle, message: "Hide snapshots checkbox was not hittable")

        let first = version(1, in: app)
        let middle = version(2, in: app)
        let third = version(3, in: app)
        XCTAssertTrue(third.waitForExistence(timeout: 5), "Expanded evolution versions did not appear")

        first.click()
        XCUIElement.perform(withKeyModifiers: .command) {
            third.click()
        }
        XCTAssertTrue(first.isSelected)
        XCTAssertFalse(middle.isSelected)
        XCTAssertTrue(third.isSelected)
        XCTAssertTrue(app.staticTexts["wip1.txt"].waitForExistence(timeout: 5), "Selected versions were not diffed")
        let comparisonBanner = app.descendants(matching: .any)[AID.Evolog.comparisonBanner].firstMatch
        XCTAssertTrue(comparisonBanner.waitForExistence(timeout: 5), "Comparison header did not appear")
        XCTAssertLessThan(
            comparisonBanner.frame.minY - toggle.frame.maxY,
            40,
            "Comparison header should stay at the top of the diff pane"
        )
        let reverse = app.descendants(matching: .any)[AID.Evolog.reverseComparison].firstMatch
        XCTAssertTrue(reverse.waitForExistence(timeout: 5), "Reverse comparison button did not appear")
        XCTAssertTrue(reverse.isEnabled)
        reverse.click()
        XCTAssertTrue(app.staticTexts["wip1.txt"].waitForExistence(timeout: 5), "Reversed versions were not diffed")

        XCUIElement.perform(withKeyModifiers: .command) {
            middle.click()
        }
        XCTAssertTrue(first.isSelected)
        XCTAssertTrue(middle.isSelected)
        XCTAssertFalse(third.isSelected)

        first.click()
        XCUIElement.perform(withKeyModifiers: .shift) {
            third.click()
        }
        XCTAssertTrue(first.isSelected)
        XCTAssertFalse(middle.isSelected)
        XCTAssertTrue(third.isSelected)
    }

    private func openEvolution(in app: XCUIApplication) -> XCUIElement {
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        rightClickCenter(rows.element(boundBy: 0))
        let showEvolution = app.menuItems["Show evolution…"].firstMatch
        XCTAssertTrue(showEvolution.waitForExistence(timeout: 5), "Show evolution menu item did not appear")
        let copyChangeId = app.menuItems["Copy Change ID"].firstMatch
        let copyCommitId = app.menuItems["Copy Commit ID"].firstMatch
        XCTAssertTrue(copyChangeId.exists, "Copy Change ID menu item did not appear")
        XCTAssertTrue(copyCommitId.exists, "Copy Commit ID menu item did not appear")
        XCTAssertLessThan(showEvolution.frame.minY, copyChangeId.frame.minY)
        XCTAssertLessThan(copyChangeId.frame.minY, copyCommitId.frame.minY)
        showEvolution.click()

        let toggle = app.descendants(matching: .any)[AID.Evolog.hideSnapshots].firstMatch
        XCTAssertTrue(toggle.waitForExistence(timeout: 5), "Hide snapshots checkbox did not appear")
        return toggle
    }

    private func version(_ index: Int, in app: XCUIApplication) -> XCUIElement {
        app.descendants(matching: .any)[AID.Evolog.version(index)].firstMatch
    }

    private func collapsedSnapshotRun(in app: XCUIApplication) -> XCUIElement {
        app.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier BEGINSWITH %@", "evolog.snapshotRun."))
            .firstMatch
    }
}
