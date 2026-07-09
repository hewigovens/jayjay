import AppKit
import SwiftUI
import WebKit

/// Sandboxed preview container: content only loads through `RepoPreviewSchemeHandler`, never `file:`, keeping WebKit's read-access root scoped to one repo checkout instead of the whole filesystem.
struct PreviewWebView: NSViewRepresentable {
    /// Markdown is pre-rendered HTML (content may not match disk, e.g. a historical revision) with `location` only for resolving asset references; HTML loads the on-disk file directly so its content and assets share the same containment check.
    enum ContentSource: Equatable {
        case renderedHTML(String, location: RepoPreviewLocation?)
        case file(RepoPreviewLocation)

        var blockerSourceKind: PreviewContentBlocker.SourceKind {
            switch self {
                case .renderedHTML: return .sanitizedHTML
                case .file: return .rawHTML
            }
        }
    }

    let source: ContentSource

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.defaultWebpagePreferences.allowsContentJavaScript = false
        configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
        configuration.websiteDataStore = .nonPersistent()
        configuration.setURLSchemeHandler(context.coordinator.schemeHandler, forURLScheme: RepoPreviewSchemeHandler.scheme)

        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = context.coordinator
        webView.setValue(false, forKey: "drawsBackground")
        // Raw HTML must not load until the blocker is installed or confirmed unavailable; see Coordinator.load.
        context.coordinator.prepareBlocker(for: configuration)
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

        func prepareBlocker(for configuration: WKWebViewConfiguration) {
            PreviewContentBlocker.apply(to: configuration) { [weak self] ruleList in
                self?.blockerDidResolve(ruleList != nil ? .ready : .unavailable)
            }
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
                    // No other guard exists against remote subresources here, so refuse to render the raw file.
                    webView.loadHTMLString(PreviewWebView.blockerUnavailableHTML, baseURL: nil)
            }
        }

        private func performLoad(_ source: ContentSource, in webView: WKWebView) {
            switch source {
                case let .renderedHTML(html, location):
                    guard let location else {
                        schemeHandler.setRoot(nil)
                        webView.loadHTMLString(html, baseURL: nil)
                        return
                    }
                    // The handler is registered once at configuration time, but each load may target a different repo, so the root it serves from is swapped out per load instead.
                    schemeHandler.setRoot(location.root)
                    webView.loadHTMLString(html, baseURL: location.documentDirectoryURL)
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
            // Meta refresh in raw HTML never reports .linkActivated, so file: must be cancelled for every navigation type or the linkActivated gate below alone would let hostile markup address arbitrary local paths.
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

    /// Same-document scroll requires matching scheme/host/path plus a fragment change — scheme alone would let a link silently navigate to a sibling file under the same root; everything else, including all `file:` URLs, is foreign and cancelled.
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

    /// JayJay-authored, references no remote resources, so it needs no content blocking of its own.
    static let blockerUnavailableHTML = """
    <html><body style="display:flex;align-items:center;justify-content:center;height:100vh;margin:0;
    font:13px -apple-system;color:#8a8a8a;">Preview unavailable: content filtering could not be enabled.</body></html>
    """
}
