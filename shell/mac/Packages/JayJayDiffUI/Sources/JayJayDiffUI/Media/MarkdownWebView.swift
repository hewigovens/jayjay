import AppKit
import SwiftUI
import WebKit

struct MarkdownWebView: NSViewRepresentable {
    let html: String
    let baseURL: URL?

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.defaultWebpagePreferences.allowsContentJavaScript = false
        configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
        configuration.websiteDataStore = .nonPersistent()

        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = context.coordinator
        webView.setValue(false, forKey: "drawsBackground")
        context.coordinator.load(html, baseURL: baseURL, in: webView)
        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        context.coordinator.load(html, baseURL: baseURL, in: webView)
    }

    final class Coordinator: NSObject, WKNavigationDelegate {
        private var loadedHTML: String?
        private var loadedBaseURL: URL?
        private var loadedFileURL: URL?

        deinit {
            removeLoadedFile()
        }

        func load(_ html: String, baseURL: URL?, in webView: WKWebView) {
            guard loadedHTML != html || loadedBaseURL != baseURL else { return }
            loadedHTML = html
            loadedBaseURL = baseURL
            removeLoadedFile()

            guard let baseURL, baseURL.isFileURL else {
                webView.loadHTMLString(html, baseURL: baseURL)
                return
            }

            do {
                let contentBaseURL = MarkdownWebView.directoryBaseURL(for: baseURL)
                let fileHTML = MarkdownWebView.htmlForFileLoad(html, baseURL: contentBaseURL)
                let fileURL = try MarkdownWebView.writeTemporaryHTMLFile(fileHTML)
                loadedFileURL = fileURL
                webView.loadFileURL(
                    fileURL,
                    allowingReadAccessTo: MarkdownWebView.readAccessURL(
                        htmlURL: fileURL,
                        contentBaseURL: contentBaseURL
                    )
                )
            } catch {
                webView.loadHTMLString(html, baseURL: baseURL)
            }
        }

        private func removeLoadedFile() {
            guard let loadedFileURL else { return }
            try? FileManager.default.removeItem(at: loadedFileURL)
            self.loadedFileURL = nil
        }

        func webView(
            _ webView: WKWebView,
            decidePolicyFor navigationAction: WKNavigationAction,
            decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
        ) {
            guard navigationAction.navigationType == .linkActivated,
                  let url = navigationAction.request.url
            else {
                decisionHandler(.allow)
                return
            }

            if url.scheme == "about", url.fragment != nil {
                decisionHandler(.allow)
            } else if ["http", "https", "mailto"].contains(url.scheme?.lowercased() ?? "") {
                NSWorkspace.shared.open(url)
                decisionHandler(.cancel)
            } else {
                decisionHandler(.cancel)
            }
        }
    }

    static func htmlForFileLoad(_ html: String, baseURL: URL) -> String {
        let baseTag = "<base href=\"\(escapeHTMLAttribute(baseURL.absoluteString))\">\n"
        guard let head = html.range(of: "<head>", options: .caseInsensitive) else {
            return baseTag + html
        }
        var html = html
        html.insert(contentsOf: "\n\(baseTag)", at: head.upperBound)
        return html
    }

    static func directoryBaseURL(for url: URL) -> URL {
        let standardized = url.standardizedFileURL
        return standardized.hasDirectoryPath ? standardized : standardized.deletingLastPathComponent()
    }

    static func readAccessURL(htmlURL: URL, contentBaseURL: URL) -> URL {
        commonAncestor(
            htmlURL.deletingLastPathComponent().standardizedFileURL,
            contentBaseURL.standardizedFileURL
        )
    }

    static func writeTemporaryHTMLFile(_ html: String) throws -> URL {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("JayJayMarkdownPreview", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let url = directory
            .appendingPathComponent(UUID().uuidString)
            .appendingPathExtension("html")
        try html.write(to: url, atomically: true, encoding: .utf8)
        return url
    }

    private static func commonAncestor(_ lhs: URL, _ rhs: URL) -> URL {
        let lhsComponents = lhs.pathComponents
        let rhsComponents = rhs.pathComponents
        var commonComponents: [String] = []

        for (left, right) in zip(lhsComponents, rhsComponents) {
            guard left == right else { break }
            commonComponents.append(left)
        }

        if commonComponents.isEmpty {
            return lhs
        }
        return URL(fileURLWithPath: NSString.path(withComponents: commonComponents), isDirectory: true)
    }

    private static func escapeHTMLAttribute(_ value: String) -> String {
        value
            .replacingOccurrences(of: "&", with: "&amp;")
            .replacingOccurrences(of: "\"", with: "&quot;")
            .replacingOccurrences(of: "<", with: "&lt;")
            .replacingOccurrences(of: ">", with: "&gt;")
    }
}
