import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class MarkdownDiffViewTests: XCTestCase {
    func testSmallMarkdownUsesWebRenderer() {
        XCTAssertEqual(
            MarkdownDiffView.renderPlan(for: "# Title\n\nBody"),
            .web("# Title\n\nBody")
        )
    }

    func testLargeMarkdownAvoidsRichRenderer() {
        let markdown = String(repeating: "notebook cell\n", count: 30000)

        let plan = MarkdownDiffView.renderPlan(for: markdown)

        guard case let .plainPreview(preview, truncated) = plan else {
            return XCTFail("expected very large markdown to avoid WebKit rich renderer")
        }
        XCTAssertTrue(truncated)
        XCTAssertEqual(preview.count, MarkdownDiffView.largePlainPreviewCharacterLimit)
    }

    func testEmptyMarkdownShowsEmptyState() {
        XCTAssertEqual(MarkdownDiffView.renderPlan(for: nil), .empty)
        XCTAssertEqual(MarkdownDiffView.renderPlan(for: ""), .empty)
    }

    func testMarkdownHTMLRendersCommonBlocks() {
        let html = renderMarkdownHtml(markdown: """
        # Title

        - One
        - **Two**

        | Name | Value |
        | --- | --- |
        | Code | `ok` |

        ```swift
        print("hi")
        ```
        """)

        XCTAssertTrue(html.contains("<h1 id=\"title\">Title</h1>"))
        XCTAssertTrue(html.contains("<ul>"))
        XCTAssertTrue(html.contains("<strong>Two</strong>"))
        XCTAssertTrue(html.contains("<table>"))
        XCTAssertTrue(html.contains("<code class=\"language-swift\">print(&quot;hi&quot;)"))
    }

    func testMarkdownHTMLRendersGFMExtensions() {
        let html = renderMarkdownHtml(markdown: """
        - [x] Done
        - [ ] Todo

        ~~removed~~

        https://example.com/path?a=1&b=2, user@example.com, and <https://example.org>.
        """)

        XCTAssertTrue(html.contains("<ul class=\"contains-task-list\">"))
        XCTAssertTrue(html.contains("<input type=\"checkbox\" disabled checked> Done"))
        XCTAssertTrue(html.contains("<input type=\"checkbox\" disabled> Todo"))
        XCTAssertTrue(html.contains("<del>removed</del>"))
        XCTAssertTrue(html.contains("href=\"https://example.com/path?a=1&amp;b=2\""))
        XCTAssertTrue(html.contains(">https://example.com/path?a=1&amp;b=2</a>,"))
        XCTAssertTrue(html.contains("href=\"mailto:user@example.com\""))
        XCTAssertTrue(html.contains("href=\"https://example.org\""))
    }

    func testMarkdownHTMLRendersImageSyntaxAndRawImageTags() {
        let html = renderMarkdownHtml(markdown: """
        ![Diagram & flow](images/flow.png "Flow")

        <p><img src="./screens/a.png" alt="A &amp; B" title="Preview" onerror="alert(1)"></p>
        <p align="center"><img src=images/raw-flow.png alt=Raw></p>
        <p align="center">
          <img src="docs/imgs/home.webp" width="100%" alt="JayJay - DAG graph and side-by-side diff">
        </p>

        ![Data](data:image/png;base64,abc123)
        """)

        XCTAssertTrue(html.contains("<img src=\"images/flow.png\" alt=\"Diagram &amp; flow\" title=\"Flow\""))
        XCTAssertTrue(html.contains("<img src=\"./screens/a.png\" alt=\"A &amp; B\" title=\"Preview\""))
        XCTAssertTrue(html.contains("<img src=\"images/raw-flow.png\" alt=\"Raw\""))
        XCTAssertTrue(html.contains("<img src=\"docs/imgs/home.webp\" alt=\"JayJay - DAG graph and side-by-side diff\""))
        XCTAssertTrue(html.contains("<img src=\"data:image/png;base64,abc123\" alt=\"Data\""))
        XCTAssertFalse(html.contains("onerror"))
        XCTAssertFalse(html.contains("&lt;p align=&quot;center&quot;&gt;"))
    }

    func testMarkdownWebViewFileLoadAddsBaseURL() throws {
        let html = "<!doctype html><html><head><meta charset=\"utf-8\"></head><body><img src=\"docs/imgs/home.webp\"></body></html>"
        let baseURL = URL(fileURLWithPath: "/tmp/jayjay repo", isDirectory: true)

        let fileHTML = MarkdownWebView.htmlForFileLoad(html, baseURL: baseURL)

        XCTAssertTrue(fileHTML.contains("<base href=\"file:///tmp/jayjay%20repo/\">"))
        XCTAssertLessThan(
            try XCTUnwrap(fileHTML.range(of: "<base")?.lowerBound),
            try XCTUnwrap(fileHTML.range(of: "<meta")?.lowerBound)
        )
    }

    func testMarkdownHTMLRejectsUnsafeImageSources() {
        let html = renderMarkdownHtml(markdown: """
        ![bad](javascript:alert(1))
        <img src="file:///etc/passwd" alt="secret">
        <img src="../secret.png" alt="secret">
        <img src="data:image/svg+xml;base64,PHN2Zz4=" alt="svg">
        """)

        XCTAssertFalse(html.contains("<img src=\"javascript:alert(1)\""))
        XCTAssertFalse(html.contains("<img src=\"file:///etc/passwd\""))
        XCTAssertFalse(html.contains("<img src=\"../secret.png\""))
        XCTAssertFalse(html.contains("<img src=\"data:image/svg+xml"))
        XCTAssertTrue(html.contains("&lt;img src=&quot;file:///etc/passwd&quot; alt=&quot;secret&quot;&gt;"))
    }

    func testMarkdownHTMLEscapesRawHTMLAndUnsafeLinks() {
        let html = renderMarkdownHtml(markdown: """
        <script>alert(1)</script>

        [bad](javascript:alert(1)) [good](https://example.com?a=1&b=2)
        """)

        XCTAssertFalse(html.contains("<script>alert(1)</script>"))
        XCTAssertTrue(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"))
        XCTAssertFalse(html.contains("href=\"javascript:alert(1)\""))
        XCTAssertTrue(html.contains("href=\"https://example.com?a=1&amp;b=2\""))
    }
}
