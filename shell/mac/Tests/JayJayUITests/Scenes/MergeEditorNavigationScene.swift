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
        let first = app.element(AID.Conflict.hunkCard(0))
        XCTAssertTrue(first.waitForExistence(timeout: 20))
        let window = app.windows.firstMatch
        window.coordinate(withNormalizedOffset: CGVector(dx: 1, dy: 1))
            .withOffset(CGVector(dx: -2, dy: -2))
            .press(
                forDuration: 0.1,
                thenDragTo: window.coordinate(withNormalizedOffset: .zero)
                    .withOffset(CGVector(dx: 1022, dy: 672))
            )
        XCTAssertTrue(waitForSelection(first))
        let leftScroll = app.scrollViews.containing(.textView, identifier: AID.Conflict.editorSource("left")).firstMatch
        XCTAssertTrue(app.element(AID.Conflict.editorSource("left") + ".lineNumbers").exists)
        XCTAssertTrue(app.element(AID.Conflict.editorSource("right") + ".lineNumbers").exists)
        let rightBar = app.scrollViews.containing(.textView, identifier: AID.Conflict.editorSource("right")).firstMatch.scrollBars.firstMatch
        let hunks = app.scrollViews[AID.Conflict.editorHunkList]
        XCTAssertTrue(hunks.exists)
        try scroll(leftScroll, by: -1800, following: [rightBar, hunks.scrollBars.firstMatch])
        try scroll(hunks, by: 1800, following: [rightBar])
        XCTAssertEqual(first.value as? String, "Selected")
        clickCenter(app.buttons[AID.Conflict.hunkUse(0, "left")])
        let second = app.element(AID.Conflict.hunkCard(1))
        XCTAssertTrue(waitForSelection(second))
        XCTAssertTrue(waitForVisibleActions(for: second, in: hunks, app: app))
        app.staticTexts["Conflict 2"].click()
        app.typeKey(.downArrow, modifierFlags: .option)
        XCTAssertTrue(waitForSelection(app.element(AID.Conflict.hunkCard(2))))
        app.typeKey(.upArrow, modifierFlags: .option)
        XCTAssertTrue(waitForSelection(second))

        clickCenter(app.element(AID.Conflict.editorRaw))
        let result = app.textViews[AID.Conflict.editorResult]
        XCTAssertTrue(result.waitForExistence(timeout: 10))
        XCTAssertTrue(app.element(AID.Conflict.editorResult + ".lineNumbers").exists)
        let resultScroll = app.scrollViews.containing(.textView, identifier: AID.Conflict.editorResult).firstMatch
        try scroll(leftScroll, by: -400, following: [rightBar])
        try scroll(resultScroll, by: 400, following: [rightBar])
        result.click()
        result.typeKey(.upArrow, modifierFlags: .command)
        paste("// manually edited result\n")
        XCTAssertTrue((result.value as? String)?.hasPrefix("// manually edited result\n") == true)
        clickCenter(app.buttons[AID.ExternalTool.baseVisibility])
        XCTAssertTrue(app.textViews[AID.Conflict.editorSource("base")].waitForExistence(timeout: 5))
        XCTAssertTrue(app.element(AID.Conflict.editorSource("base") + ".lineNumbers").exists)

        clickCenter(app.element(AID.Conflict.editorHunks))
        XCTAssertTrue(second.waitForExistence(timeout: 5))
        XCTAssertEqual(second.value as? String, "Selected")
        XCTAssertTrue(waitForVisibleActions(for: second, in: hunks, app: app))
        clickCenter(app.buttons[AID.Conflict.hunkUse(1, "right")])
        let third = app.element(AID.Conflict.hunkCard(2))
        XCTAssertTrue(waitForSelection(third))
        clickCenter(app.buttons[AID.Conflict.editorCancel])
    }

    private func scroll(
        _ viewport: XCUIElement,
        by deltaY: CGFloat,
        following followers: [XCUIElement],
        file: StaticString = #filePath,
        line: UInt = #line
    ) throws {
        XCTAssertTrue(viewport.waitForExistence(timeout: 5), "Scroll viewport did not appear", file: file, line: line)
        let bars = [viewport.scrollBars.firstMatch] + followers
        let initialValues = try bars.map { try XCTUnwrap($0.value as? Double, "Expected a numeric scroll position", file: file, line: line) }
        viewport.scroll(byDeltaX: 0, deltaY: deltaY)
        let moved = NSPredicate { _, _ in
            zip(bars, initialValues).allSatisfy { bar, initial in
                guard let current = bar.value as? Double else { return false }
                return current != initial
            }
        }
        XCTAssertTrue(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: moved, object: nil)], timeout: 5) == .completed,
            "Scroll viewport and followers did not move from \(initialValues); current positions: \(bars.map { String(describing: $0.value) })",
            file: file,
            line: line
        )
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
