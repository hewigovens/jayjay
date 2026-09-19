import XCTest

final class MergeEditorNavigationScene: ExternalToolSceneBase {
    override class var additionalLaunchArguments: [String] {
        let root = fixtureRoot.appendingPathComponent("external-tool", isDirectory: true)
        return ["tool", "merge"] + ["left", "base", "right", "output"].map {
            root.appendingPathComponent("merge-scroll-\($0).swift").path
        }
    }

    func testSynchronizedScrollingAndHunkNavigation() throws {
        let app = try XCTUnwrap(app)
        let first = app.descendants(matching: .any)[AID.Conflict.hunkCard(0)]
        XCTAssertTrue(first.waitForExistence(timeout: 20))
        XCTAssertEqual(first.value as? String, "Selected")
        let window = app.windows.firstMatch
        window.coordinate(withNormalizedOffset: CGVector(dx: 1, dy: 1))
            .withOffset(CGVector(dx: -2, dy: -2))
            .press(
                forDuration: 0.1,
                thenDragTo: window.coordinate(withNormalizedOffset: .zero)
                    .withOffset(CGVector(dx: 1022, dy: 672))
            )
        let left = app.textViews[AID.Conflict.editorSource("left")]
        XCTAssertTrue(app.descendants(matching: .any)[AID.Conflict.editorSource("left") + ".lineNumbers"].exists)
        XCTAssertTrue(app.descendants(matching: .any)[AID.Conflict.editorSource("right") + ".lineNumbers"].exists)
        let rightScroll = app.scrollViews.containing(.textView, identifier: AID.Conflict.editorSource("right")).firstMatch
        let rightBar = rightScroll.scrollBars.firstMatch
        let hunks = app.scrollViews[AID.Conflict.editorHunkList]
        XCTAssertTrue(hunks.exists)
        let hunkBar = hunks.scrollBars.firstMatch
        let initialRight = String(describing: rightBar.value)
        let initialHunks = String(describing: hunkBar.value)
        left.scroll(byDeltaX: 0, deltaY: -1800)
        XCTAssertNotEqual(String(describing: rightBar.value), initialRight)
        XCTAssertNotEqual(String(describing: hunkBar.value), initialHunks)
        let beforeHunkScroll = String(describing: rightBar.value)
        hunks.scroll(byDeltaX: 0, deltaY: 1800)
        XCTAssertNotEqual(String(describing: rightBar.value), beforeHunkScroll)
        XCTAssertEqual(first.value as? String, "Selected")
        clickCenter(app.buttons[AID.Conflict.hunkUse(0, "left")])
        let second = app.descendants(matching: .any)[AID.Conflict.hunkCard(1)]
        XCTAssertTrue(waitForSelection(second))
        XCTAssertTrue(waitForVisibleActions(for: second, in: hunks, app: app))
        app.staticTexts["Conflict 2"].click()
        app.typeKey(.downArrow, modifierFlags: .option)
        XCTAssertTrue(waitForSelection(app.descendants(matching: .any)[AID.Conflict.hunkCard(2)]))
        app.typeKey(.upArrow, modifierFlags: .option)
        XCTAssertTrue(waitForSelection(second))

        clickCenter(app.descendants(matching: .any)[AID.Conflict.editorRaw])
        let result = app.textViews[AID.Conflict.editorResult]
        XCTAssertTrue(result.waitForExistence(timeout: 10))
        XCTAssertTrue(app.descendants(matching: .any)[AID.Conflict.editorResult + ".lineNumbers"].exists)
        let beforeScroll = String(describing: rightBar.value)
        left.scroll(byDeltaX: 0, deltaY: -400)
        XCTAssertNotEqual(String(describing: rightBar.value), beforeScroll)
        let beforeResultScroll = String(describing: rightBar.value)
        result.scroll(byDeltaX: 0, deltaY: 400)
        XCTAssertNotEqual(String(describing: rightBar.value), beforeResultScroll)
        result.click()
        result.typeKey(.upArrow, modifierFlags: .command)
        paste("// manually edited result\n")
        XCTAssertTrue((result.value as? String)?.hasPrefix("// manually edited result\n") == true)
        clickCenter(app.buttons[AID.ExternalTool.baseVisibility])
        XCTAssertTrue(app.textViews[AID.Conflict.editorSource("base")].waitForExistence(timeout: 5))
        XCTAssertTrue(app.descendants(matching: .any)[AID.Conflict.editorSource("base") + ".lineNumbers"].exists)

        clickCenter(app.descendants(matching: .any)[AID.Conflict.editorHunks])
        XCTAssertTrue(second.waitForExistence(timeout: 5))
        XCTAssertEqual(second.value as? String, "Selected")
        XCTAssertTrue(waitForVisibleActions(for: second, in: hunks, app: app))
        clickCenter(app.buttons[AID.Conflict.hunkUse(1, "right")])
        let third = app.descendants(matching: .any)[AID.Conflict.hunkCard(2)]
        XCTAssertTrue(waitForSelection(third))
        clickCenter(app.buttons[AID.Conflict.editorCancel])
    }

    private func waitForVisibleActions(for card: XCUIElement, in viewport: XCUIElement, app: XCUIApplication) -> Bool {
        let accept = app.buttons[AID.Conflict.hunkUse(1, "right")]
        let visible = NSPredicate { _, _ in
            viewport.frame.contains(card.frame) && viewport.frame.contains(accept.frame)
        }
        return XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: visible, object: card)], timeout: 5) == .completed
    }

    private func waitForSelection(_ card: XCUIElement) -> Bool {
        XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", "Selected"), object: card)], timeout: 5) == .completed
    }
}
