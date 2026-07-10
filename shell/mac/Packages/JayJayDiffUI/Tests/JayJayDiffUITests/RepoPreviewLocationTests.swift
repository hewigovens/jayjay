@testable import JayJayDiffUI
import XCTest

final class RepoPreviewLocationTests: XCTestCase {
    private let root = URL(fileURLWithPath: "/tmp/jayjay repo", isDirectory: true)

    func testDocumentURLAddressesFileByPathWithinRoot() throws {
        let location = try XCTUnwrap(RepoPreviewLocation(root: root, relativePath: "docs/guide/index.html"))

        XCTAssertEqual(location.documentURL.absoluteString, "jayjay-preview://content/docs/guide/index.html")
        XCTAssertEqual(location.documentDirectoryURL.absoluteString, "jayjay-preview://content/docs/guide/")
    }

    func testRootLevelFileUsesSchemeRootAsBase() throws {
        let location = try XCTUnwrap(RepoPreviewLocation(root: root, relativePath: "README.md"))

        XCTAssertEqual(location.documentURL.absoluteString, "jayjay-preview://content/README.md")
        XCTAssertEqual(location.documentDirectoryURL, RepoPreviewSchemeHandler.baseURL)
    }

    func testRelativeAndParentRelativeReferencesResolveWithinRoot() throws {
        let base = try XCTUnwrap(RepoPreviewLocation(root: root, relativePath: "docs/index.html")).documentDirectoryURL

        XCTAssertEqual(URL(string: "img/a.png", relativeTo: base)?.absoluteURL.path, "/docs/img/a.png")
        XCTAssertEqual(URL(string: "../assets/site.css", relativeTo: base)?.absoluteURL.path, "/assets/site.css")
        // References climbing past the scheme root clamp to it, so the request still maps under the containment root.
        XCTAssertEqual(URL(string: "../../../etc/passwd", relativeTo: base)?.absoluteURL.path, "/etc/passwd")
    }

    func testSpecialCharactersInPathRoundTripThroughRequestPath() throws {
        let location = try XCTUnwrap(RepoPreviewLocation(root: root, relativePath: "docs & files/a b.html"))

        XCTAssertEqual(location.documentURL.absoluteString, "jayjay-preview://content/docs%20&%20files/a%20b.html")
        XCTAssertEqual(location.documentURL.path, "/docs & files/a b.html")
    }

    func testInteriorParentComponentsStayingInsideRootAreAccepted() {
        XCTAssertNotNil(RepoPreviewLocation(root: root, relativePath: "docs/../README.md"))
    }

    func testEscapingRelativePathIsRejected() {
        XCTAssertNil(RepoPreviewLocation(root: root, relativePath: "../outside.md"))
        XCTAssertNil(RepoPreviewLocation(root: root, relativePath: "docs/../../outside.md"))
    }

    func testEmptyDotAndNonFileInputsAreRejected() {
        XCTAssertNil(RepoPreviewLocation(root: root, relativePath: ""))
        XCTAssertNil(RepoPreviewLocation(root: root, relativePath: "."))
        XCTAssertNil(RepoPreviewLocation(root: URL(string: "https://example.com/repo")!, relativePath: "a.md"))
    }
}
