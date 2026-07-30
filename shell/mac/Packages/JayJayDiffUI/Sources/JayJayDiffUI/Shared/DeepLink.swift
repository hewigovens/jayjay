import Foundation

/// Central registry of every URL scheme and host the app produces or handles; new links declare their host here instead of minting a scheme.
public enum DeepLink {
    public static let scheme = "jayjay"

    public enum Host {
        public static let open = "open"
        public static let diffContext = "diff-context"
    }

    /// WKWebView preview content keeps a dedicated scheme: its handler is registered per web view and must not collide with the OS-registered app scheme.
    public enum Preview {
        public static let scheme = "jayjay-preview"
    }
}
