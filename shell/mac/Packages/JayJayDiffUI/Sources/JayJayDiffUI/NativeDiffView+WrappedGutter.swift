import AppKit
import JayJayCore

struct NativeDiffGutterRenderContext {
    /// Group/marker column width: three spaces over the 6pt stripe for a comfortable click target.
    static let groupColumnText = "   "

    /// Unspliced display lines — group/stripe math runs on these because every line number in play is unspliced.
    let lines: [DiffLine]
    /// Render order including embedded note rows; must match the content view's paragraphs one to one.
    let rows: [DiffRenderRow]
    let visualLineCounts: [Int]
    let font: NSFont
    let theme: DiffColors
    let gutterAttrs: [NSAttributedString.Key: Any]
    let gutterParagraphStyle: NSMutableParagraphStyle
    let maxLineDigits: Int
    let reviewModeEnabled: Bool
    let showsCheckboxColumn: Bool
    let groupIndexAtLineNumber: [Int: UInt32]
    let reviewActions: (any DiffGutterReviewActions)?
    let notedLines: Set<Int>
    /// Lines whose notes are all resolved: the marker dims to a record of past review instead of a call to action.
    let resolvedOnlyLines: Set<Int>
    let showsNoteColumn: Bool
    let groupStripeWidth: CGFloat
    let gutterHorizontalInset: CGFloat
    let gutterTrailingPadding: CGFloat
    let currentSelectedLineRange: ClosedRange<Int>?
}

extension NativeDiffGutterRenderContext {
    var blankGutterLine: NSAttributedString {
        blankGutterLine(paragraphStyle: nil)
    }

    func blankGutterLine(paragraphStyle: NSParagraphStyle?) -> NSAttributedString {
        let blankNumber = String(repeating: " ", count: maxLineDigits)
        let noteColumn = showsNoteColumn ? "  " : ""
        let checkboxColumn = showsCheckboxColumn ? "  " : ""
        var attrs = gutterAttrs
        if let paragraphStyle {
            attrs[.paragraphStyle] = paragraphStyle
        }
        return NSAttributedString(
            string: "\(Self.groupColumnText)\(noteColumn)\(checkboxColumn)\(blankNumber) \(blankNumber)\n",
            attributes: attrs
        )
    }

    /// Mirrors the content view's bubble spacing on the gutter's blank rows; without it every line after a note drifts out of alignment.
    func noteGutterParagraphStyle(spacingBefore: Bool, spacingAfter: Bool) -> NSParagraphStyle {
        let style = NSMutableParagraphStyle()
        style.setParagraphStyle(gutterParagraphStyle)
        if spacingBefore { style.paragraphSpacingBefore = DiffNoteBubbleMetrics.verticalSpacing }
        if spacingAfter { style.paragraphSpacing = DiffNoteBubbleMetrics.verticalSpacing }
        return style
    }
}

private struct GutterBuilder {
    var attributed = NSMutableAttributedString()
    var entries: [DiffGutterTextView.Entry] = []
    var lineBgColors: [NSColor] = []
    var stripeColors: [NSColor] = []

    mutating func append(
        _ line: NSAttributedString,
        style: DiffSpanStyle,
        lineNumber: Int,
        bgColor: NSColor,
        stripeColor: NSColor
    ) {
        let start = attributed.length
        attributed.append(line)
        entries.append(.init(
            style: style,
            range: NSRange(location: start, length: attributed.length - start),
            lineNumber: lineNumber
        ))
        lineBgColors.append(bgColor)
        stripeColors.append(stripeColor)
    }
}

extension NativeDiffView {
    func renderWrappedGutter(
        gutterTextView: DiffGutterTextView,
        gutterLayoutManager: DiffLayoutManager,
        context: NativeDiffGutterRenderContext
    ) -> CGFloat {
        var builder = GutterBuilder()
        var gutterWidth: CGFloat = 0

        for (index, row) in context.rows.enumerated() {
            let visualRows = index < context.visualLineCounts.count
                ? max(1, context.visualLineCounts[index])
                : 1
            guard case let .line(line, lineNumber) = row else {
                // Note rows carry a unique negative id so hover, selection, and column hit-testing all skip them.
                if case let .note(_, _, isFirst, isLast) = row {
                    for visualRow in 0 ..< visualRows {
                        builder.append(
                            context.blankGutterLine(paragraphStyle: context.noteGutterParagraphStyle(
                                spacingBefore: isFirst && visualRow == 0,
                                spacingAfter: isLast && visualRow == visualRows - 1
                            )),
                            style: .context,
                            lineNumber: -(index + 1),
                            bgColor: .clear,
                            stripeColor: .clear
                        )
                    }
                }
                continue
            }
            let bgColor = line.style == .separator
                ? context.theme.separatorBg
                : context.theme.lineBg(line)
            let stripeColor = stripeColor(for: line, lineNumber: lineNumber, context: context)

            if line.style == .separator {
                for _ in 0 ..< visualRows {
                    builder.append(
                        context.blankGutterLine,
                        style: line.style,
                        lineNumber: lineNumber,
                        bgColor: bgColor,
                        stripeColor: .clear
                    )
                }
                continue
            }

            let gutterLine = firstVisualGutterLine(
                for: line,
                lineNumber: lineNumber,
                context: context
            )
            builder.append(
                gutterLine,
                style: line.style,
                lineNumber: lineNumber,
                bgColor: bgColor,
                stripeColor: stripeColor
            )
            gutterWidth = max(
                gutterWidth,
                ceil(
                    context.gutterHorizontalInset +
                        gutterLine.size().width +
                        context.gutterTrailingPadding +
                        context.gutterHorizontalInset
                )
            )

            for _ in 1 ..< visualRows {
                builder.append(
                    context.blankGutterLine,
                    style: line.style,
                    lineNumber: lineNumber,
                    bgColor: bgColor,
                    stripeColor: stripeColor
                )
            }
        }

        if context.rows.isEmpty {
            builder.append(
                NSAttributedString(string: "\n", attributes: context.gutterAttrs),
                style: .context,
                lineNumber: 1,
                bgColor: .clear,
                stripeColor: .clear
            )
        }

        gutterLayoutManager.lineBgColors = builder.lineBgColors
        gutterLayoutManager.lineStripeColors = builder.stripeColors
        gutterLayoutManager.lineStripeX = 0
        gutterLayoutManager.lineStripeWidth = context.groupStripeWidth
        gutterTextView.textStorage?.setAttributedString(builder.attributed)
        gutterTextView.entries = builder.entries
        gutterTextView.externalSelection = context.currentSelectedLineRange
        return gutterWidth
    }

    private func stripeColor(
        for line: DiffLine,
        lineNumber: Int,
        context: NativeDiffGutterRenderContext
    ) -> NSColor {
        if context.reviewModeEnabled,
           let groupIdx = context.groupIndexAtLineNumber[lineNumber]
        {
            if context.reviewActions?.isHunkReviewed(groupIndex: groupIdx) == true {
                return NSColor.controlAccentColor
            }
            return NSColor.selectedTextBackgroundColor
        }
        // Reuse updateNSView's display lines; expandedHunkRange would re-run the diffDisplayLines FFI per line (O(n^2)).
        return groupStripeColor(
            for: line,
            groupRange: DiffGutterGrouping.expandedChangedRange(
                in: context.lines,
                containing: lineNumber ... lineNumber
            ),
            theme: context.theme
        )
    }

    private func firstVisualGutterLine(
        for line: DiffLine,
        lineNumber: Int,
        context: NativeDiffGutterRenderContext
    ) -> NSMutableAttributedString {
        // Reviewed state shows only through the group stripe's accent color; a ✓ glyph here would be a third, redundant signal.
        let gutterLine = NSMutableAttributedString(
            string: NativeDiffGutterRenderContext.groupColumnText,
            attributes: [
                .font: context.font,
                .foregroundColor: context.theme.gutterText,
                .paragraphStyle: context.gutterParagraphStyle
            ]
        )
        if context.showsNoteColumn {
            let hasNote = context.notedLines.contains(lineNumber)
            let resolvedOnly = context.resolvedOnlyLines.contains(lineNumber)
            gutterLine.append(NSAttributedString(
                string: hasNote ? "● " : "  ",
                attributes: [
                    .font: context.font,
                    .foregroundColor: resolvedOnly ? NSColor.tertiaryLabelColor : NSColor.systemOrange,
                    .paragraphStyle: context.gutterParagraphStyle
                ]
            ))
        }
        if context.showsCheckboxColumn {
            gutterLine.append(NSAttributedString(
                string: checkboxText(for: lineNumber, line: line),
                attributes: [
                    .font: context.font,
                    .foregroundColor: checkboxColor(for: lineNumber, theme: context.theme),
                    .paragraphStyle: context.gutterParagraphStyle
                ]
            ))
        }
        gutterLine.append(NSAttributedString(
            string: pad(line.oldLineNo.map(String.init) ?? "", toWidth: context.maxLineDigits),
            attributes: context.gutterAttrs
        ))
        gutterLine.append(NSAttributedString(string: " ", attributes: context.gutterAttrs))
        gutterLine.append(NSAttributedString(
            string: pad(line.newLineNo.map(String.init) ?? "", toWidth: context.maxLineDigits),
            attributes: context.gutterAttrs
        ))
        gutterLine.append(NSAttributedString(string: "\n", attributes: context.gutterAttrs))
        return gutterLine
    }
}
