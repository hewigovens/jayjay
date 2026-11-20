import XCTest

final class ImageStackScrollScene: SceneBase {
    override class var fixtureName: String {
        "image-stack"
    }

    override class var additionalLaunchArguments: [String] {
        ["-jayjay.treeFileList", "NO"]
    }

    func testPagingKeysScrollAStackWithoutTextDiffs() throws {
        let app = try XCTUnwrap(app)
        let first = fileRows(of: app).matching(identifier: AID.FileList.row("a.png")).firstMatch
        XCTAssertTrue(first.waitForExistence(timeout: 10))
        first.click()
        keyStroke("a", modifiers: .command)
        let firstHeader = app.descendants(matching: .any).matching(identifier: AID.Diff.section).firstMatch
        XCTAssertTrue(firstHeader.waitForExistence(timeout: 10))
        XCTAssertTrue(waitForHittable(firstHeader, isHittable: true))

        keyStroke(.pageDown)
        XCTAssertTrue(waitForHittable(firstHeader, isHittable: false), "Page Down did not scroll the stacked image diffs")
        keyStroke(.home)
        XCTAssertTrue(waitForHittable(firstHeader, isHittable: true), "Home did not return to the first image diff")
        keyStroke(.end)
        XCTAssertTrue(waitForHittable(firstHeader, isHittable: false), "End did not scroll the stacked image diffs")
    }
}
