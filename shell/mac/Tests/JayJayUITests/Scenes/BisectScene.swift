import XCTest

final class BisectScene: SceneBase {
    override class var fixtureName: String {
        "bisect"
    }

    func testBisectBetweenSelectsTheResultAndRestoresTheWorkingCopy() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")
        let workingCopy = try XCTUnwrap(
            rows.allElementsBoundByIndex.first { ($0.value as? String)?.contains("working copy") == true },
            "working copy row missing"
        )
        let workingCopyId = workingCopy.identifier

        let feature = try XCTUnwrap(
            rows.allElementsBoundByIndex.first { ($0.value as? String)?.contains("add feature") == true },
            "add feature row missing"
        )
        let initial = try XCTUnwrap(
            rows.allElementsBoundByIndex.first { ($0.value as? String)?.contains("initial") == true },
            "initial row missing"
        )
        feature.click()
        XCUIElement.perform(withKeyModifiers: .command) {
            initial.click()
        }
        rightClickCenter(feature)
        let between = app.menuItems["Bisect between"]
        XCTAssertTrue(between.waitForExistence(timeout: 3), "Bisect between missing")
        between.click()

        let good = app.buttons[AID.Bisect.good]
        XCTAssertTrue(good.waitForExistence(timeout: 10), "bisect never started")
        let workingCopyRows = rows.allElementsBoundByIndex.filter { ($0.value as? String)?.contains("working copy") == true }
        XCTAssertEqual(workingCopyRows.count, 1, "only the test checkout should be the working copy")
        good.click()

        let status = app.descendants(matching: .any)[AID.Bisect.status]
        let found = NSPredicate { _, _ in
            (status.value as? String)?.contains("First bad change") == true
                || (status.label as? String)?.contains("First bad change") == true
                || status.staticTexts.firstMatch.label.contains("First bad change")
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: found, object: nil)], timeout: 10),
            .completed,
            "bisect did not finish"
        )

        let selected = rows.allElementsBoundByIndex.first { $0.isSelected }
        XCTAssertTrue(
            (selected?.value as? String)?.contains("add feature") == true,
            "the result should be selected"
        )
        let restored = rows.allElementsBoundByIndex.first { $0.identifier == workingCopyId }
        XCTAssertTrue(
            (restored?.value as? String)?.contains("working copy") == true,
            "@ should be restored"
        )
    }
}
