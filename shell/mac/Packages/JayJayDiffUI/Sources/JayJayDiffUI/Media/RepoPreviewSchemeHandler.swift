import Foundation
import UniformTypeIdentifiers
import WebKit

/// Serves assets over a custom URL scheme instead of WebKit `file:` access, which shares one root per WKWebView and can widen to "/" when the repo and temp directory share no path prefix.
final class RepoPreviewSchemeHandler: NSObject, WKURLSchemeHandler {
    static let scheme = "jayjay-preview"
    /// Fixed placeholder document URL; the real root lives in `root` and is resolved per request, not from this URL.
    static let baseURL = URL(string: "\(scheme)://content/")!

    private let lock = NSLock()
    private var _root: URL?

    /// Settable because one WKWebViewConfiguration (and its scheme handler) is reused across loads that may target different directories.
    func setRoot(_ root: URL?) {
        lock.lock()
        _root = root
        lock.unlock()
    }

    private var root: URL? {
        lock.lock()
        defer { lock.unlock() }
        return _root
    }

    func webView(_ webView: WKWebView, start urlSchemeTask: WKURLSchemeTask) {
        guard let requestURL = urlSchemeTask.request.url,
              let root,
              let fileURL = Self.resolvedFileURL(forRequestPath: requestURL.path, root: root)
        else {
            urlSchemeTask.didFailWithError(URLError(.fileDoesNotExist))
            return
        }

        do {
            let data = try Data(contentsOf: fileURL)
            let response = URLResponse(
                url: requestURL,
                mimeType: Self.mimeType(forPathExtension: fileURL.pathExtension),
                expectedContentLength: data.count,
                textEncodingName: nil
            )
            urlSchemeTask.didReceive(response)
            urlSchemeTask.didReceive(data)
            urlSchemeTask.didFinish()
        } catch {
            urlSchemeTask.didFailWithError(error)
        }
    }

    func webView(_ webView: WKWebView, stop urlSchemeTask: WKURLSchemeTask) {
        // start(_:) resolves and responds synchronously, so there is nothing in flight to cancel.
    }

    /// Canonicalizes both sides (following symlinks) and requires the result to land strictly inside the canonicalized root as a regular file outside VCS internals; never falls back to an uncontained read. Request paths may contain `..` segments (inside-root ones resolve, escaping ones are rejected here).
    static func resolvedFileURL(forRequestPath path: String, root: URL) -> URL? {
        guard root.isFileURL else { return nil }
        let canonicalRoot = root.resolvingSymlinksInPath().standardizedFileURL
        let relativePath = path.hasPrefix("/") ? String(path.dropFirst()) : path
        guard !relativePath.isEmpty else { return nil }

        let candidate = canonicalRoot.appendingPathComponent(relativePath)
        let canonicalCandidate = candidate.resolvingSymlinksInPath().standardizedFileURL

        let rootPrefix = canonicalRoot.path.hasSuffix("/") ? canonicalRoot.path : canonicalRoot.path + "/"
        guard canonicalCandidate.path.hasPrefix(rootPrefix) else { return nil }

        // No legitimate preview asset lives under .jj/.git, so the repo-rooted handler never exposes VCS internals (op store, refs, credential-bearing git config) to previewed markup; checked on the canonical path so a symlink into them is also caught.
        let relativeComponents = canonicalCandidate.path.dropFirst(rootPrefix.count).split(separator: "/")
        let vcsInternals: Set<String> = [".jj", ".git"]
        guard !relativeComponents.contains(where: { vcsInternals.contains($0.lowercased()) }) else {
            return nil
        }

        guard (try? canonicalCandidate.resourceValues(forKeys: [.isRegularFileKey]))?.isRegularFile == true else {
            return nil
        }
        return canonicalCandidate
    }

    static func mimeType(forPathExtension pathExtension: String) -> String {
        guard let type = UTType(filenameExtension: pathExtension) else { return "application/octet-stream" }
        return type.preferredMIMEType ?? "application/octet-stream"
    }
}
