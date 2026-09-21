import XCTest

final class FileRenameScene: SceneBase {
    override class var fixtureName: String {
        "complex"
    }

    func testRenameRowExposesSourceAndDestinationWithoutHovering() throws {
        let app = try XCTUnwrap(app)
        let description = "Renamed from docs/guides/guide-05.md to docs/reference/guide-05.md"
        let column = app.descendants(matching: .any)[AID.FileList.column]
        let row = column.descendants(matching: .any).matching(NSPredicate(
            format: "label CONTAINS %@ OR value CONTAINS %@", description, description
        )).firstMatch
        XCTAssertTrue(row.waitForExistence(timeout: 10), "The file row must expose the complete rename")
        XCTAssertTrue(row.isHittable)
    }
}
