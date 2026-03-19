import SwiftUI
import WebKit

struct MonacoDiffView: NSViewRepresentable {
    let path: String
    let original: String
    let modified: String

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontScale) private var fontScale
    @Environment(AppSettings.self) private var settings

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> WKWebView {
        let contentController = WKUserContentController()
        contentController.add(context.coordinator, name: "monacoReady")

        let configuration = WKWebViewConfiguration()
        configuration.userContentController = contentController
        configuration.preferences.setValue(true, forKey: "developerExtrasEnabled")

        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.setValue(false, forKey: "drawsBackground")
        webView.navigationDelegate = context.coordinator
        context.coordinator.webView = webView

        if let rootURL = Bundle.main.resourceURL?.appendingPathComponent("WebDiff") {
            let indexURL = rootURL.appendingPathComponent("index.html")
            webView.loadFileURL(indexURL, allowingReadAccessTo: rootURL)
        }

        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        let payload = Payload(
            path: path,
            original: original,
            modified: modified,
            theme: settings.diffTheme.resolved(for: colorScheme),
            fontSize: max(11, Int((12 * fontScale).rounded())),
            renderSideBySide: true,
            wordWrap: "on"
        )
        context.coordinator.setPayload(payload)
        context.coordinator.pushPayloadIfReady()
    }

    final class Coordinator: NSObject, WKScriptMessageHandler, WKNavigationDelegate {
        weak var webView: WKWebView?
        var isReady = false
        private var pendingPayload: Payload?

        func userContentController(
            _ userContentController: WKUserContentController,
            didReceive message: WKScriptMessage
        ) {
            if message.name == "monacoReady" {
                isReady = true
                pushPayloadIfReady()
            }
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            pushPayloadIfReady()
        }

        func pushPayloadIfReady() {
            guard isReady, let payload = pendingPayload, let webView else {
                return
            }
            guard let data = try? JSONEncoder().encode(payload),
                  let json = String(data: data, encoding: .utf8) else {
                return
            }
            webView.evaluateJavaScript("window.renderDiff(\(json));", completionHandler: nil)
        }

        fileprivate func setPayload(_ payload: Payload) {
            pendingPayload = payload
        }
    }

    fileprivate struct Payload: Codable {
        let path: String
        let original: String
        let modified: String
        let theme: String
        let fontSize: Int
        let renderSideBySide: Bool
        let wordWrap: String
    }
}
