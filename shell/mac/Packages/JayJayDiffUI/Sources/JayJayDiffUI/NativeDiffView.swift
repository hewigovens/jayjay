import AppKit
import JayJayCore
import SwiftUI

public struct NativeDiffView: NSViewRepresentable {
    public typealias Coordinator = NativeDiffContextCoordinator

    public let diff: FileDiff
    public var gutterActions: (any DiffGutterContextActions)?
    /// Passed as a property (not pulled through `gutterActions`) so SwiftUI re-runs `updateNSView` when the note list changes.
    public var reviewNotes: [DiffReviewNoteSummary]
    /// Precomputed once per loaded diff by the owner; `updateNSView` re-runs on every observed change and the display-line/group FFI is O(diff bytes).
    public var displayLines: [DiffLine]?
    public var displayGroups: [ChangeGroup]?
    public var reserveNoteColumn: Bool
    public var compactGutterWidth: Bool
    /// Shows a dedicated +/- column for compact merge-hunk comparisons where the change type must never be confused with editable text.
    public var showsChangeMarkers: Bool
    public var onExpandContext: ((DiffContextExpansionRequest) -> Void)?
    public var resetSelectionGeneration: UInt64
    /// Enables constant-time selection refreshes when the owner increments this value for every rendered-content change.
    public var contentGeneration: UInt64?
    public var revealFeedback: DiffContextRevealFeedback?
    /// When set, the view sizes to its full content: inner scrolling is disabled and every laid-out height change is reported so the host can match its frame.
    public var onContentHeightChanged: ((CGFloat) -> Void)?

    @Environment(\.colorScheme) var colorScheme
    @Environment(\.diffFontSize) var fontSize
    @Environment(\.diffFontFamily) var fontFamily
    @Environment(\.accessibilityReduceMotion) var reduceMotion

    public init(
        diff: FileDiff,
        gutterActions: (any DiffGutterContextActions)? = nil,
        reviewNotes: [DiffReviewNoteSummary] = [],
        displayLines: [DiffLine]? = nil,
        displayGroups: [ChangeGroup]? = nil,
        reserveNoteColumn: Bool = false,
        compactGutterWidth: Bool = false,
        showsChangeMarkers: Bool = false,
        onExpandContext: ((DiffContextExpansionRequest) -> Void)? = nil,
        resetSelectionGeneration: UInt64 = 0,
        contentGeneration: UInt64? = nil,
        revealFeedback: DiffContextRevealFeedback? = nil,
        onContentHeightChanged: ((CGFloat) -> Void)? = nil
    ) {
        self.diff = diff
        self.gutterActions = gutterActions
        self.reviewNotes = reviewNotes
        self.displayLines = displayLines
        self.displayGroups = displayGroups
        self.reserveNoteColumn = reserveNoteColumn
        self.compactGutterWidth = compactGutterWidth
        self.showsChangeMarkers = showsChangeMarkers
        self.onExpandContext = onExpandContext
        self.resetSelectionGeneration = resetSelectionGeneration
        self.contentGeneration = contentGeneration
        self.revealFeedback = revealFeedback
        self.onContentHeightChanged = onContentHeightChanged
    }
}
