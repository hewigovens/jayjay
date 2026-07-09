import Foundation

/// A previewed file addressed as a containment root (the repo checkout) plus its forward-slash relative path; deriving scheme URLs from the root rather than the file's own directory lets parent-relative references like `../assets/site.css` resolve anywhere inside the checkout while `RepoPreviewSchemeHandler` still rejects anything outside it.
public struct RepoPreviewLocation: Equatable {
    public let root: URL
    public let relativePath: String

    /// Fails for non-file roots and paths that are empty or lexically escape the root, so a hostile relative path never becomes a previewable address; the scheme handler re-checks every request with symlink canonicalization.
    public init?(root: URL, relativePath: String) {
        guard root.isFileURL, !relativePath.isEmpty else { return nil }
        let standardizedRoot = root.standardizedFileURL
        let rootPrefix = standardizedRoot.path.hasSuffix("/") ? standardizedRoot.path : standardizedRoot.path + "/"
        let filePath = standardizedRoot.appendingPathComponent(relativePath).standardizedFileURL.path
        guard filePath.hasPrefix(rootPrefix) else { return nil }
        self.root = standardizedRoot
        self.relativePath = relativePath
    }

    /// Scheme URL of the file itself; component-wise appending percent-encodes each segment and drops empty ones so the handler's percent-decoded request path round-trips exactly.
    var documentURL: URL {
        relativePath.split(separator: "/").reduce(RepoPreviewSchemeHandler.baseURL) {
            $0.appendingPathComponent(String($1))
        }
    }

    /// Scheme URL of the file's directory within the root, used as the document base so `assets/x.png` and `../assets/x.png` both resolve against the root and stay containment-checked.
    var documentDirectoryURL: URL {
        documentURL.deletingLastPathComponent()
    }
}
