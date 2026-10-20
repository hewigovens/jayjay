import JayJayCore
@testable import JayJayDiffUI
import WebKit
import XCTest

@MainActor
final class MermaidPreviewTests: XCTestCase {
    func testMermaidBlocksRenderWhilePageJavaScriptStaysDisabled() async throws {
        let coordinator = PreviewWebView.Coordinator()
        let webView = coordinator.makeWebView()
        let html = renderMarkdownHtml(markdown: "```mermaid\nflowchart LR\n A --> B\n```") +
            "<script>document.body.dataset.pageScript = \"ran\"</script>"

        coordinator.load(.renderedHTML(html, location: nil), in: webView)

        let script = """
        document.querySelector('.mermaid-diagram svg.ondine') !== null &&
        document.querySelector('pre') === null &&
        document.body.dataset.pageScript === undefined
        """
        let deadline = Date().addingTimeInterval(10)
        while Date() < deadline {
            if let result = try? await webView.evaluateJavaScript(script, in: nil, contentWorld: .defaultClient),
               result as? Bool == true
            {
                return
            }
            try await Task.sleep(for: .milliseconds(100))
        }
        XCTFail("Mermaid did not render with page JavaScript disabled")
    }
}
