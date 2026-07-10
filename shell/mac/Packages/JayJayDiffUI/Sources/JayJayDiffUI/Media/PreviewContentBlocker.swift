import WebKit

/// Network-level backstop for preview WKWebViews. `jayjay-markdown` already refuses to emit `<img>` markup for http/https (or any other absolute-scheme) sources, but that sanitizer only runs on rendered Markdown — `HTMLDiffView` loads a working-copy HTML file's own markup unfiltered, so this rule list is the only guard against a remote `<img>`/`<link>` in that path. Content rule lists only ever see http(s) requests, so `jayjay-preview://` and `data:` loads are unaffected without needing an explicit allow rule.
@MainActor
enum PreviewContentBlocker {
    static let identifier = "JayJayPreviewRemoteBlock"

    static let ruleListJSON = """
    [{"trigger":{"url-filter":"^https?://"},"action":{"type":"block"}}]
    """

    /// Readiness of the shared rule-list compilation.
    enum State: Equatable {
        case pending
        case ready
        case unavailable
    }

    /// How exposed a preview's markup is to unsanitized remote subresources.
    enum SourceKind {
        case sanitizedHTML
        case rawHTML
    }

    enum LoadDecision: Equatable {
        case proceed
        case wait
        case failClosed
    }

    /// `.rawHTML` has no guard but this blocker, so it must wait or fail closed; `.sanitizedHTML` is already scrubbed by the renderer and may always proceed.
    static func loadDecision(for sourceKind: SourceKind, blockerState: State) -> LoadDecision {
        guard sourceKind == .rawHTML else { return .proceed }
        switch blockerState {
            case .ready: return .proceed
            case .pending: return .wait
            case .unavailable: return .failClosed
        }
    }

    private static var cachedRuleList: WKContentRuleList?
    private static var compilationFailed = false
    private static var isCompiling = false
    private static var waiters: [(WKContentRuleList?) -> Void] = []

    /// Joins the single shared compilation for `identifier` instead of racing a fresh compile per preview, and attaches the eventual result to `configuration`; `completion` lets a caller gate a pending load on it.
    static func apply(to configuration: WKWebViewConfiguration, completion: ((WKContentRuleList?) -> Void)? = nil) {
        if let cachedRuleList {
            configuration.userContentController.add(cachedRuleList)
            completion?(cachedRuleList)
            return
        }
        if compilationFailed {
            completion?(nil)
            return
        }

        waiters.append { ruleList in
            if let ruleList {
                configuration.userContentController.add(ruleList)
            }
            completion?(ruleList)
        }
        guard !isCompiling else { return }
        isCompiling = true

        WKContentRuleListStore.default().compileContentRuleList(
            forIdentifier: identifier,
            encodedContentRuleList: ruleListJSON
        ) { ruleList, error in
            // Completion queue is unspecified; hop back to the main actor before touching shared state.
            Task { @MainActor in
                isCompiling = false
                if let ruleList {
                    cachedRuleList = ruleList
                } else {
                    compilationFailed = true
                    if let error {
                        NSLog("JayJay: preview content rule list compilation failed: \(error)")
                    }
                }
                let resolved = waiters
                waiters = []
                resolved.forEach { $0(ruleList) }
            }
        }
    }
}
