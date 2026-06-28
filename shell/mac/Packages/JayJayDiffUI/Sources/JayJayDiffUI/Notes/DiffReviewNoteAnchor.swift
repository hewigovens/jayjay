import JayJayCore

public struct DiffReviewNoteAnchor: Hashable {
    public let groupIndex: UInt32
    public let displayLine: UInt32
    public let side: DiffSide
    public let line: UInt32
    public let excerpt: String
    public let context: [String]

    public init(
        groupIndex: UInt32,
        displayLine: UInt32,
        side: DiffSide,
        line: UInt32,
        excerpt: String,
        context: [String]
    ) {
        self.groupIndex = groupIndex
        self.displayLine = displayLine
        self.side = side
        self.line = line
        self.excerpt = excerpt
        self.context = context
    }
}
