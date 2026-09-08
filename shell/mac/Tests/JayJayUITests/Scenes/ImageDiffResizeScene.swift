import XCTest

final class ImageDiffResizeScene: SceneBase {
    override class var fixtureName: String {
        "image-diff"
    }

    func testResizeImagesAndRestoreEqualWidths() throws {
        let app = try XCTUnwrap(app)
        let file = fileRows(of: app).firstMatch
        XCTAssertTrue(file.waitForExistence(timeout: 10))
        file.click()

        let divider = app.descendants(matching: .any).matching(identifier: AID.Diff.section)
            .matching(NSPredicate(format: "label == %@", "Image comparison divider")).firstMatch
        XCTAssertTrue(divider.waitForExistence(timeout: 10))
        let beforeImage = app.images["Before image"]
        let afterImage = app.images["After image"]
        XCTAssertTrue(beforeImage.waitForExistence(timeout: 5))
        XCTAssertTrue(afterImage.waitForExistence(timeout: 5))
        let beforeWidth = beforeImage.frame.width
        let afterWidth = afterImage.frame.width
        let initialX = divider.frame.midX
        let grip = divider.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
        grip.press(forDuration: 0.2, thenDragTo: grip.withOffset(CGVector(dx: 40, dy: 0)))
        XCTAssertEqual(divider.frame.midX, initialX + 40, accuracy: 4)
        XCTAssertGreaterThan(beforeImage.frame.width, beforeWidth)

        let movedGrip = divider.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
        movedGrip.press(forDuration: 0.2, thenDragTo: movedGrip.withOffset(CGVector(dx: -80, dy: 0)))
        XCTAssertEqual(divider.frame.midX, initialX - 40, accuracy: 4)
        XCTAssertGreaterThan(afterImage.frame.width, afterWidth)

        divider.doubleClick()
        XCTAssertEqual(divider.frame.midX, initialX, accuracy: 2)
    }
}
