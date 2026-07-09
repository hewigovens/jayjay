@testable import JayJayDiffUI
import XCTest

final class PreviewWebViewLinkTests: XCTestCase {
    func testAboutFragmentOnSameDocumentIsAllowed() {
        let document = URL(string: "about:blank")!
        XCTAssertEqual(
            PreviewWebView.linkDecision(for: URL(string: "about:blank#intro")!, documentURL: document),
            .allow
        )
    }

    func testExternalSchemesOpenExternally() {
        for raw in ["https://example.com", "http://example.com", "mailto:a@b.c"] {
            XCTAssertEqual(PreviewWebView.linkDecision(for: URL(string: raw)!, documentURL: nil), .openExternally)
        }
    }

    func testCustomSchemeFragmentOnSameDocumentIsAllowed() {
        let document = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/")!
        let url = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/#intro")!
        XCTAssertEqual(PreviewWebView.linkDecision(for: url, documentURL: document), .allow)
    }

    func testCustomSchemeFileFragmentOnSameDocumentIsAllowed() {
        let document = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/index.html")!
        let url = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/index.html#intro")!
        XCTAssertEqual(PreviewWebView.linkDecision(for: url, documentURL: document), .allow)
    }

    func testCustomSchemeFragmentToADifferentFileIsCancelled() {
        // Must not silently navigate away from the previewed file to a sibling under the same root.
        let document = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/index.html")!
        let url = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/other.html#intro")!
        XCTAssertEqual(PreviewWebView.linkDecision(for: url, documentURL: document), .cancel)
    }

    func testCustomSchemeFragmentWithNoKnownDocumentIsCancelled() {
        let url = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/index.html#intro")!
        XCTAssertEqual(PreviewWebView.linkDecision(for: url, documentURL: nil), .cancel)
    }

    func testCustomSchemeWithoutFragmentIsCancelled() {
        let document = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/other.md")!
        let url = URL(string: "\(RepoPreviewSchemeHandler.scheme)://content/other.md")!
        XCTAssertEqual(PreviewWebView.linkDecision(for: url, documentURL: document), .cancel)
    }

    func testFileURLIsForeignAndCancelled() {
        XCTAssertEqual(PreviewWebView.linkDecision(for: URL(string: "file:///etc/passwd")!, documentURL: nil), .cancel)
        XCTAssertEqual(
            PreviewWebView.linkDecision(for: URL(string: "file:///etc/passwd#x")!, documentURL: nil),
            .cancel
        )
    }

    func testUnknownSchemeIsCancelled() {
        XCTAssertEqual(PreviewWebView.linkDecision(for: URL(string: "javascript:alert(1)")!, documentURL: nil), .cancel)
    }
}
