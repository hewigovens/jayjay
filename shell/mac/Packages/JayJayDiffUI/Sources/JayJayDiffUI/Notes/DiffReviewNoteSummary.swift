import JayJayCore

public struct DiffReviewNoteSummary: Hashable {
    public let id: String
    public let body: String
    public let side: DiffSide?
    public let line: UInt32?
    public let excerpt: String?
    /// Stale notes (diff changed under the anchor) keep their gutter marker but are never expanded into the diff body.
    public let isStale: Bool
    /// Resolved notes keep a dimmed gutter marker as a record of the review, but never expand and only offer Delete.
    public let isResolved: Bool

    public init(
        id: String,
        body: String,
        side: DiffSide? = nil,
        line: UInt32? = nil,
        excerpt: String? = nil,
        isStale: Bool = false,
        isResolved: Bool = false
    ) {
        self.id = id
        self.body = body
        self.side = side
        self.line = line
        self.excerpt = excerpt
        self.isStale = isStale
        self.isResolved = isResolved
    }
}
