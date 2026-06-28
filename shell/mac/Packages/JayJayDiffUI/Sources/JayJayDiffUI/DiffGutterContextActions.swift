import JayJayCore

public enum DiffGutterCheckboxState {
    case selected
    case unselected
}

public protocol DiffGutterContextActions {
    var currentSelectedLineRange: ClosedRange<Int>? { get }

    func didSelectLines(_ lineRange: ClosedRange<Int>)
}

public protocol DiffGutterSelectionActions: DiffGutterContextActions {
    func selectFile()
    func selectChangeGroup(_ lineRange: ClosedRange<Int>)
    func lineCheckboxState(for lineNumber: Int) -> DiffGutterCheckboxState?
    func toggleLineCheckbox(_ lineNumber: Int)
}

public protocol DiffGutterEditActions: DiffGutterContextActions {
    var canOpenDiffEdit: Bool { get }
    var canAbandonSelectedLines: Bool { get }

    func openDiffEdit()
    func abandonSelectedLines(in lineRange: ClosedRange<Int>)
}

public protocol DiffGutterReviewActions: DiffGutterContextActions {
    /// Whether the per-group review column renders; false where review state doesn't apply (e.g., compare/interdiff mode against another revision).
    var reviewModeEnabled: Bool { get }

    func isHunkReviewed(groupIndex: UInt32) -> Bool
    func toggleHunkReviewed(groupIndex: UInt32)
}

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

public protocol DiffGutterNoteActions: DiffGutterContextActions {
    var reviewNotesEnabled: Bool { get }

    func activeNotes(anchor: DiffReviewNoteAnchor) -> [DiffReviewNoteSummary]
    func addNote(anchor: DiffReviewNoteAnchor)
    func editNote(id: String)
    func deleteNote(id: String)
    func resolveNote(id: String)
}
