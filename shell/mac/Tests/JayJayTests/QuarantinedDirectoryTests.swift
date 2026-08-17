@testable import JayJay
import XCTest

final class QuarantinedDirectoryTests: XCTestCase {
    private var workspace: URL!

    override func setUpWithError() throws {
        workspace = FileManager.default.temporaryDirectory
            .appendingPathComponent("quarantine-\(UUID().uuidString)/feature", isDirectory: true)
        try FileManager.default.createDirectory(at: workspace, withIntermediateDirectories: true)
        try Data("contents".utf8).write(to: workspace.appendingPathComponent("file.txt"))
    }

    override func tearDownWithError() throws {
        try? FileManager.default.removeItem(at: workspace.deletingLastPathComponent())
    }

    func testDeleteOnlySeesTheCapturedDirectoryNotAReplacement() throws {
        let identity = try QuarantinedDirectory.identity(path: workspace.path)
        let quarantined = try QuarantinedDirectory.capture(
            path: workspace.path,
            expectedIdentity: identity
        )
        XCTAssertFalse(FileManager.default.fileExists(atPath: workspace.path))

        // Another process reuses the path while the async operation runs; the delete must not touch it.
        let replacement = workspace.appendingPathComponent("precious.txt")
        try FileManager.default.createDirectory(at: workspace, withIntermediateDirectories: false)
        try Data("keep me".utf8).write(to: replacement)

        try quarantined.delete()

        XCTAssertTrue(FileManager.default.fileExists(atPath: replacement.path))
        XCTAssertFalse(FileManager.default.fileExists(atPath: quarantined.quarantineURL.path))
    }

    func testRestorePutsTheDirectoryBack() throws {
        let identity = try QuarantinedDirectory.identity(path: workspace.path)
        let quarantined = try QuarantinedDirectory.capture(
            path: workspace.path,
            expectedIdentity: identity
        )
        try quarantined.restore()

        let contents = try String(contentsOf: workspace.appendingPathComponent("file.txt"), encoding: .utf8)
        XCTAssertEqual(contents, "contents")
        XCTAssertFalse(FileManager.default.fileExists(atPath: quarantined.quarantineURL.path))
    }

    func testCaptureRejectsEmptyRelativeAndRootPaths() throws {
        for path in ["", "relative-workspace", "/"] {
            XCTAssertThrowsError(try QuarantinedDirectory.identity(path: path), path)
        }
    }

    func testCaptureRestoresAReplacementWhenItsIdentityChanged() throws {
        let identity = try QuarantinedDirectory.identity(path: workspace.path)
        try FileManager.default.removeItem(at: workspace)
        try FileManager.default.createDirectory(at: workspace, withIntermediateDirectories: false)
        let replacement = workspace.appendingPathComponent("precious.txt")
        try Data("keep me".utf8).write(to: replacement)

        XCTAssertThrowsError(
            try QuarantinedDirectory.capture(path: workspace.path, expectedIdentity: identity)
        )

        XCTAssertEqual(try String(contentsOf: replacement, encoding: .utf8), "keep me")
    }
}
