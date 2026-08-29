import XCTest

final class EvologHideSnapshotsScene: SceneBase {
    override class var fixtureName: String {
        "evolog-hide-snapshots"
    }

    func testToggleHideSnapshotsAndExpandCollapsedRun() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        rightClickCenter(rows.element(boundBy: 0))
        let showEvolution = app.menuItems["Show evolution…"].firstMatch
        XCTAssertTrue(showEvolution.waitForExistence(timeout: 5), "Show evolution menu item did not appear")
        showEvolution.click()

        let toggle = app.descendants(matching: .any)[AID.Evolog.hideSnapshots].firstMatch
        XCTAssertTrue(toggle.waitForExistence(timeout: 5), "Hide snapshots checkbox did not appear")

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

    private func collapsedSnapshotRun(in app: XCUIApplication) -> XCUIElement {
        app.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier BEGINSWITH %@", "evolog.snapshotRun."))
            .firstMatch
    }
}
