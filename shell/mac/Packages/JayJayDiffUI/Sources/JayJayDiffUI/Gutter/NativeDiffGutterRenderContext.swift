import AppKit
import JayJayCore

struct NativeDiffGutterRenderContext {
    /// Group/marker column width: three spaces over the 6pt stripe for a comfortable click target.
    static let groupColumnText = DiffGutterMetrics.groupColumnText

    /// What gets rendered: the display lines and their row/wrap expansion.
    struct Content {
        /// Unspliced display lines — group/stripe math runs on these because every line number in play is unspliced.
        let lines: [DiffLine]
        /// Render order including embedded note rows; must match the content view's paragraphs one to one.
        let rows: [DiffRenderRow]
        let visualLineCounts: [Int]
    }

    /// How gutter text is drawn: font, colors, and shared attributes.
    struct Style {
        let font: NSFont
        let theme: DiffColors
        let gutterAttrs: [NSAttributedString.Key: Any]
        let gutterParagraphStyle: NSMutableParagraphStyle
        let maxLineDigits: Int
    }

    /// Column geometry and which optional columns are present.
    struct Layout {
        let groupStripeWidth: CGFloat
        let gutterHorizontalInset: CGFloat
        let gutterTrailingPadding: CGFloat
        let showsCheckboxColumn: Bool
        let showsNoteColumn: Bool
        let showsChangeMarkers: Bool

        init(
            groupStripeWidth: CGFloat,
            gutterHorizontalInset: CGFloat,
            gutterTrailingPadding: CGFloat,
            showsCheckboxColumn: Bool,
            showsNoteColumn: Bool,
            showsChangeMarkers: Bool = false
        ) {
            self.groupStripeWidth = groupStripeWidth
            self.gutterHorizontalInset = gutterHorizontalInset
            self.gutterTrailingPadding = gutterTrailingPadding
            self.showsCheckboxColumn = showsCheckboxColumn
            self.showsNoteColumn = showsNoteColumn
            self.showsChangeMarkers = showsChangeMarkers
        }
    }

    /// Review-mode state: hunk review stripes, note markers, and selection.
    struct Review {
        let reviewModeEnabled: Bool
        let groupIndexAtLineNumber: [Int: UInt32]
        let reviewActions: (any DiffGutterReviewActions)?
        let notedLines: Set<Int>
        /// Lines whose notes are all resolved: the marker dims to a record of past review instead of a call to action.
        let resolvedOnlyLines: Set<Int>
        let currentSelectedLineRange: ClosedRange<Int>?
    }

    let content: Content
    let style: Style
    let layout: Layout
    let review: Review
}

extension NativeDiffGutterRenderContext {
    var blankGutterLine: NSAttributedString {
        blankGutterLine(paragraphStyle: nil)
    }

    func blankGutterLine(paragraphStyle: NSParagraphStyle?) -> NSAttributedString {
        let blankNumber = String(repeating: " ", count: style.maxLineDigits)
        let noteColumn = layout.showsNoteColumn ? "  " : ""
        let checkboxColumn = layout.showsCheckboxColumn ? "  " : ""
        let changeColumn = layout.showsChangeMarkers ? "  " : ""
        var attrs = style.gutterAttrs
        if let paragraphStyle {
            attrs[.paragraphStyle] = paragraphStyle
        }
        return NSAttributedString(
            string: "\(Self.groupColumnText)\(noteColumn)\(checkboxColumn)\(changeColumn)\(blankNumber) \(blankNumber)\n",
            attributes: attrs
        )
    }

    /// Mirrors the content view's bubble spacing on the gutter's blank rows; without it every line after a note drifts out of alignment.
    func noteGutterParagraphStyle(spacingBefore: Bool, spacingAfter: Bool) -> NSParagraphStyle {
        let noteStyle = NSMutableParagraphStyle()
        noteStyle.setParagraphStyle(style.gutterParagraphStyle)
        if spacingBefore {
            noteStyle.paragraphSpacingBefore = DiffNoteBubbleMetrics.verticalSpacing
        }
        if spacingAfter {
            noteStyle.paragraphSpacing = DiffNoteBubbleMetrics.verticalSpacing
        }
        return noteStyle
    }
}
