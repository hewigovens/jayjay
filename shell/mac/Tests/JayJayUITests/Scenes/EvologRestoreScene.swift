import XCTest

final class EvologRestoreScene: SceneBase {
    override class var fixtureName: String {
        "evolog-restore"
    }

    func testRestoreVersionReplacesWorkingCopyTree() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        rightClickCenter(rows.element(boundBy: 0))
        let showEvolution = app.menuItems["Show evolution…"].firstMatch
        XCTAssertTrue(showEvolution.waitForExistence(timeout: 5), "Show evolution menu item did not appear")
        showEvolution.click()

        let older = app.descendants(matching: .any)[AID.Evolog.version(1)].firstMatch
        XCTAssertTrue(older.waitForExistence(timeout: 5), "Older evolution version did not appear")
        older.rightClick()

        let restore = app.menuItems["Restore this version"].firstMatch
        XCTAssertTrue(restore.waitForExistence(timeout: 5), "Restore menu item did not appear")
        XCTAssertTrue(restore.isEnabled)
        let copyCommand = app.menuItems["Copy ‘jj restore’ command"].firstMatch
        XCTAssertTrue(copyCommand.waitForExistence(timeout: 5), "Copy restore command should remain as a secondary action")
        restore.click()

        let restored = try XCTUnwrap(fixtureURL?.appendingPathComponent("wip1.txt"))
        var content: String?
        let deadline = Date().addingTimeInterval(10)
        while Date() < deadline {
            content = try? String(contentsOf: restored, encoding: .utf8)
            if content?.trimmingCharacters(in: .whitespacesAndNewlines) == "restored content" {
                break
            }
            RunLoop.current.run(until: Date().addingTimeInterval(0.2))
        }
        XCTAssertEqual(
            content?.trimmingCharacters(in: .whitespacesAndNewlines),
            "restored content",
            "Working copy did not revert to the restored version"
        )
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG did not refresh after restore")
    }
}
