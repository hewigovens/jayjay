import AppKit
import SwiftUI
import WebKit

/// Keeps preview assets inside the selected repository through a custom URL scheme.
struct PreviewWebView: NSViewRepresentable {
    /// Historical Markdown uses rendered HTML; location supplies only its asset base.
    enum ContentSource: Equatable {
        case renderedHTML(String, location: RepoPreviewLocation?)
        case file(RepoPreviewLocation)

        var blockerSourceKind: PreviewContentBlocker.SourceKind {
            switch self {
                case .renderedHTML: .sanitizedHTML
                case .file: .rawHTML
            }
        }
    }

    let source: ContentSource

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> WKWebView {
        let webView = context.coordinator.makeWebView()
        context.coordinator.load(source, in: webView)
        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        context.coordinator.load(source, in: webView)
    }

    @MainActor
    final class Coordinator: NSObject, WKNavigationDelegate {
        let schemeHandler = RepoPreviewSchemeHandler()
        private var loadedSource: ContentSource?
        private var blockerState: PreviewContentBlocker.State = .pending
        private var pendingLoad: (source: ContentSource, webView: WKWebView)?

        func makeWebView() -> WKWebView {
            let configuration = WKWebViewConfiguration()
            configuration.defaultWebpagePreferences.allowsContentJavaScript = false
            configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
            configuration.websiteDataStore = .nonPersistent()
            configuration.setURLSchemeHandler(schemeHandler, forURLScheme: RepoPreviewSchemeHandler.scheme)

            let webView = WKWebView(frame: .zero, configuration: configuration)
            webView.navigationDelegate = self
            webView.allowsMagnification = true
            webView.setValue(false, forKey: "drawsBackground")
            PreviewContentBlocker.apply(to: configuration) { [weak self] ruleList in
                self?.blockerDidResolve(ruleList != nil ? .ready : .unavailable)
            }
            return webView
        }

        private func blockerDidResolve(_ state: PreviewContentBlocker.State) {
            blockerState = state
            guard let pendingLoad else { return }
            self.pendingLoad = nil
            performLoad(pendingLoad.source, in: pendingLoad.webView)
        }

        func load(_ source: ContentSource, in webView: WKWebView) {
            guard loadedSource != source else { return }
            loadedSource = source

            switch PreviewContentBlocker.loadDecision(for: source.blockerSourceKind, blockerState: blockerState) {
                case .proceed:
                    performLoad(source, in: webView)
                case .wait:
                    pendingLoad = (source, webView)
                case .failClosed:
                    // Raw HTML has no sanitizer to backstop a missing content blocker.
                    webView.loadHTMLString(PreviewWebView.blockerUnavailableHTML, baseURL: nil)
            }
        }

        private func performLoad(_ source: ContentSource, in webView: WKWebView) {
            switch source {
                case let .renderedHTML(html, location):
                    schemeHandler.setRoot(location?.root)
                    webView.loadHTMLString(html, baseURL: location?.documentDirectoryURL)
                case let .file(location):
                    schemeHandler.setRoot(location.root)
                    webView.load(URLRequest(url: location.documentURL))
            }
        }

        func webView(
            _ webView: WKWebView,
            decidePolicyFor navigationAction: WKNavigationAction,
            decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
        ) {
            // Meta refresh bypasses the linkActivated check, so reject file URLs for every navigation type.
            if navigationAction.request.url?.isFileURL == true {
                decisionHandler(.cancel)
                return
            }
            guard navigationAction.navigationType == .linkActivated,
                  let url = navigationAction.request.url
            else {
                decisionHandler(.allow)
                return
            }

            switch PreviewWebView.linkDecision(for: url, documentURL: webView.url) {
                case .allow:
                    decisionHandler(.allow)
                case .openExternally:
                    NSWorkspace.shared.open(url)
                    decisionHandler(.cancel)
                case .cancel:
                    decisionHandler(.cancel)
            }
        }
    }

    enum LinkDecision: Equatable {
        case allow
        case openExternally
        case cancel
    }

    /// Allow fragments only within the current document; other local URLs must stay blocked.
    static func linkDecision(for url: URL, documentURL: URL?) -> LinkDecision {
        if isSameDocumentFragmentLink(url, documentURL: documentURL) {
            return .allow
        }
        if ["http", "https", "mailto"].contains(url.scheme?.lowercased() ?? "") {
            return .openExternally
        }
        return .cancel
    }

    private static func isSameDocumentFragmentLink(_ url: URL, documentURL: URL?) -> Bool {
        guard url.fragment != nil, let documentURL else { return false }
        return url.scheme == documentURL.scheme && url.host == documentURL.host && url.path == documentURL.path
    }

    static let blockerUnavailableHTML = """
    <html><body style="display:flex;align-items:center;justify-content:center;height:100vh;margin:0;
    font:13px -apple-system;color:#8a8a8a;">Preview unavailable: content filtering could not be enabled.</body></html>
    """
}
