@testable import JayJayDiffUI
import XCTest

final class RepoPreviewSchemeHandlerTests: XCTestCase {
    private var tempDir: URL!
    private var root: URL!

    override func setUpWithError() throws {
        tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("RepoPreviewSchemeHandlerTests-\(UUID().uuidString)", isDirectory: true)
        root = tempDir.appendingPathComponent("root", isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        try? FileManager.default.removeItem(at: tempDir)
    }

    private func write(_ contents: String, at relativePath: String) throws -> URL {
        let url = root.appendingPathComponent(relativePath)
        try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        try contents.write(to: url, atomically: true, encoding: .utf8)
        return url
    }

    func testFileInsideRootResolves() throws {
        _ = try write("hello", at: "image.png")

        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/image.png", root: root)

        XCTAssertEqual(resolved?.lastPathComponent, "image.png")
    }

    func testMimeTypeInferredFromExtension() {
        XCTAssertEqual(RepoPreviewSchemeHandler.mimeType(forPathExtension: "png"), "image/png")
        XCTAssertEqual(RepoPreviewSchemeHandler.mimeType(forPathExtension: "svg"), "image/svg+xml")
        XCTAssertEqual(RepoPreviewSchemeHandler.mimeType(forPathExtension: "unknownext"), "application/octet-stream")
    }

    func testParentRelativePathInsideRootResolves() throws {
        let direct = try write("body {}", at: "assets/site.css")
        _ = try write("<html></html>", at: "docs/index.html")

        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/docs/../assets/site.css", root: root)

        XCTAssertEqual(resolved?.path, direct.resolvingSymlinksInPath().standardizedFileURL.path)
    }

    func testParentRelativePathThroughMissingDirectoryStaysContained() throws {
        let direct = try write("body {}", at: "assets/site.css")

        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/missing/../assets/site.css", root: root)

        XCTAssertEqual(resolved?.path, direct.resolvingSymlinksInPath().standardizedFileURL.path)
    }

    func testParentRelativePathEscapingRootIsRejected() throws {
        let outsideFile = tempDir.appendingPathComponent("outside.txt")
        try "secret".write(to: outsideFile, atomically: true, encoding: .utf8)
        _ = try write("<html></html>", at: "docs/index.html")

        XCTAssertNil(RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/docs/../../outside.txt", root: root))
        XCTAssertNil(RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/docs/..", root: root))
    }

    func testNonFileRootIsRejected() {
        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(
            forRequestPath: "/image.png",
            root: URL(string: "https://example.com/root")!
        )

        XCTAssertNil(resolved)
    }

    func testDocumentURLPathRoundTripsThroughHandler() throws {
        let direct = try write("<html></html>", at: "docs & files/a b.html")
        let location = try XCTUnwrap(RepoPreviewLocation(root: root, relativePath: "docs & files/a b.html"))

        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: location.documentURL.path, root: root)

        XCTAssertEqual(resolved?.path, direct.resolvingSymlinksInPath().standardizedFileURL.path)
    }

    func testParentDirectoryTraversalIsRejected() throws {
        _ = try write("secret", at: "../outside.txt")

        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/../outside.txt", root: root)

        XCTAssertNil(resolved)
    }

    func testPercentEncodedTraversalIsRejected() throws {
        _ = try write("secret", at: "../outside.txt")

        // `URL.path` already percent-decodes `%2F`/`%2e`, so this is the exact string the handler receives from `URLSchemeTask.request.url.path`.
        let requestURL = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/..%2Foutside.txt")!
        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: requestURL.path, root: root)

        XCTAssertNil(resolved)
    }

    func testSymlinkInsideRootPointingOutsideRootIsRejected() throws {
        let outsideFile = tempDir.appendingPathComponent("secret.txt")
        try "top secret".write(to: outsideFile, atomically: true, encoding: .utf8)
        let symlink = root.appendingPathComponent("escape-link")
        try FileManager.default.createSymbolicLink(at: symlink, withDestinationURL: outsideFile)

        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/escape-link", root: root)

        XCTAssertNil(resolved)
    }

    func testVCSInternalsAreRejectedEvenThoughInsideRoot() throws {
        _ = try write("secret-op-store", at: ".jj/repo/config.toml")
        _ = try write("[credential] helper=store", at: ".git/config")
        _ = try write("nested", at: "vendor/.git/config")

        XCTAssertNil(RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/.jj/repo/config.toml", root: root))
        XCTAssertNil(RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/.git/config", root: root))
        XCTAssertNil(RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/.GIT/config", root: root))
        XCTAssertNil(RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/vendor/.git/config", root: root))
        XCTAssertNil(RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/docs/../.git/config", root: root))
    }

    func testSymlinkIntoVCSInternalsIsRejected() throws {
        let target = try write("[credential] helper=store", at: ".git/config")
        let symlink = root.appendingPathComponent("innocent.css")
        try FileManager.default.createSymbolicLink(at: symlink, withDestinationURL: target)

        XCTAssertNil(RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/innocent.css", root: root))
    }

    func testOtherDotDirectoriesInsideRootStillResolve() throws {
        let direct = try write("logo", at: ".github/logo.png")

        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/.github/logo.png", root: root)

        XCTAssertEqual(resolved?.path, direct.resolvingSymlinksInPath().standardizedFileURL.path)
    }

    func testNonExistentFileIsRejected() {
        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/missing.png", root: root)

        XCTAssertNil(resolved)
    }

    func testDirectoryRequestIsRejected() throws {
        let subdirectory = root.appendingPathComponent("subdir", isDirectory: true)
        try FileManager.default.createDirectory(at: subdirectory, withIntermediateDirectories: true)

        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/subdir", root: root)

        XCTAssertNil(resolved)
    }

    func testRootRequestIsRejected() {
        let resolved = RepoPreviewSchemeHandler.resolvedFileURL(forRequestPath: "/", root: root)

        XCTAssertNil(resolved)
    }
}
