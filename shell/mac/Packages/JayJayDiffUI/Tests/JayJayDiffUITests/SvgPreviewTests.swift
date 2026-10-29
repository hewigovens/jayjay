import AppKit
@testable import JayJayDiffUI
import SwiftUI
import WebKit
import XCTest

@MainActor
final class SvgPreviewTests: XCTestCase {
    func testSvgContentCannotExecuteScriptsOrBreakOutOfTheImage() async throws {
        let url = "https://127.0.0.1:49152/preview"
        let payloads = [
            """
            <svg xmlns="http://www.w3.org/2000/svg" width="120" height="60">
            <script>document.body.dataset.previewAttack = 'ran'; fetch('\(url)')</script>
            <image href="\(url)" width="120" height="60"/>
            <rect width="120" height="60" fill="red"/>
            </svg>
            """,
            """
            </svg><img src="\(url)" onerror="document.body.dataset.previewAttack = 'ran'">
            <script>document.body.dataset.previewAttack = 'ran'</script><svg>
            """
        ]
        for payload in payloads {
            let (host, webView) = try await preview(payload)
            defer { host.removeFromSuperview() }
            try await Task.sleep(for: .milliseconds(300))
            let scriptRan = try await webView.evaluateJavaScript(
                "document.body.dataset.previewAttack === 'ran'", in: nil, contentWorld: .defaultClient
            )
            XCTAssertEqual(scriptRan as? Bool, false)
            XCTAssertEqual(webView.url?.scheme, "about")
            let activeMarkup = try await webView.evaluateJavaScript(
                "document.querySelector('script, iframe, form, svg') !== null", in: nil, contentWorld: .defaultClient
            )
            XCTAssertEqual(activeMarkup as? Bool, false)
        }
    }

    func testSvgPreviewPreservesIntrinsicDimensionsAndDownscalesToFit() async throws {
        for attributes in ["width='120' height='60'", "viewBox='0 0 120 60'", "width='100%' height='100%' viewBox='0 0 120 60'"] {
            let svg = "<svg xmlns='http://www.w3.org/2000/svg' \(attributes)><rect width='120' height='60' fill='red'/></svg>"
            let (host, webView) = try await preview(svg)
            defer { host.removeFromSuperview() }
            let deadline = Date().addingTimeInterval(5)
            var size: [Double]?
            while Date() < deadline {
                size = try? await webView.evaluateJavaScript(
                    "(() => { const image = document.querySelector('img'); return image && image.complete && image.naturalWidth > 0 ? [image.getBoundingClientRect().width, image.getBoundingClientRect().height] : null })()",
                    in: nil, contentWorld: .defaultClient
                ) as? [Double]
                if let size, size[0] > 0 {
                    break
                }
                try await Task.sleep(for: .milliseconds(20))
            }
            let loaded = try XCTUnwrap(size)
            XCTAssertEqual(loaded[0], 120, accuracy: 1)
            XCTAssertEqual(loaded[1], 60, accuracy: 1)
            host.frame.size = NSSize(width: 60, height: 30)
            host.layoutSubtreeIfNeeded()
            try await Task.sleep(for: .milliseconds(100))
            let scaled = try await webView.evaluateJavaScript(
                "(() => { const r = document.querySelector('img').getBoundingClientRect(); return [r.width, r.height] })()",
                in: nil, contentWorld: .defaultClient
            ) as? [Double]
            XCTAssertEqual(try XCTUnwrap(scaled)[0], 60, accuracy: 1)
            XCTAssertEqual(try XCTUnwrap(scaled)[1], 30, accuracy: 1)
        }
    }

    private func preview(_ svg: String) async throws -> (NSHostingView<SvgWebView>, WKWebView) {
        let host = NSHostingView(rootView: SvgWebView(svg: svg))
        host.frame = NSRect(x: 0, y: 0, width: 300, height: 200)
        host.layoutSubtreeIfNeeded()
        let deadline = Date().addingTimeInterval(10)
        while Date() < deadline {
            if let webView = webView(in: host),
               await (try? webView.evaluateJavaScript("document.readyState", in: nil, contentWorld: .defaultClient)) as? String == "complete"
            {
                return (host, webView)
            }
            try await Task.sleep(for: .milliseconds(20))
        }
        XCTFail("WebKit did not finish loading the SVG preview")
        throw NSError(domain: "SvgPreviewTests", code: 1)
    }

    private func webView(in view: NSView) -> WKWebView? {
        if let webView = view as? WKWebView {
            return webView
        }
        return view.subviews.lazy.compactMap { self.webView(in: $0) }.first
    }
}
