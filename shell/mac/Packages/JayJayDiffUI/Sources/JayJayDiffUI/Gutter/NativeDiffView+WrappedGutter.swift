import AppKit
import JayJayCore

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

        for (index, row) in context.content.rows.enumerated() {
            let visualRows = index < context.content.visualLineCounts.count
                ? max(1, context.content.visualLineCounts[index])
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
                ? context.style.theme.separatorBg
                : context.style.theme.lineBg(line)
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
                    context.layout.gutterHorizontalInset +
                        gutterLine.size().width +
                        context.layout.gutterTrailingPadding +
                        context.layout.gutterHorizontalInset
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

        if context.content.rows.isEmpty {
            builder.append(
                NSAttributedString(string: "\n", attributes: context.style.gutterAttrs),
                style: .context,
                lineNumber: 1,
                bgColor: .clear,
                stripeColor: .clear
            )
        }

        gutterLayoutManager.lineBgColors = builder.lineBgColors
        gutterLayoutManager.lineStripeColors = builder.stripeColors
        gutterLayoutManager.lineStripeX = 0
        gutterLayoutManager.lineStripeWidth = context.layout.groupStripeWidth
        gutterTextView.textStorage?.setAttributedString(builder.attributed)
        gutterTextView.entries = builder.entries
        gutterTextView.externalSelection = context.review.currentSelectedLineRange
        return gutterWidth
    }

    private func stripeColor(
        for line: DiffLine,
        lineNumber: Int,
        context: NativeDiffGutterRenderContext
    ) -> NSColor {
        if context.review.reviewModeEnabled,
           let groupIdx = context.review.groupIndexAtLineNumber[lineNumber]
        {
            if context.review.reviewActions?.isHunkReviewed(groupIndex: groupIdx) == true {
                return NSColor.controlAccentColor
            }
            return NSColor.selectedTextBackgroundColor
        }
        // Reuse updateNSView's display lines; expandedHunkRange would re-run the diffDisplayLines FFI per line (O(n^2)).
        return groupStripeColor(
            for: line,
            groupRange: DiffGutterGrouping.expandedChangedRange(
                in: context.content.lines,
                containing: lineNumber ... lineNumber
            ),
            theme: context.style.theme
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
                .font: context.style.font,
                .foregroundColor: context.style.theme.gutterText,
                .paragraphStyle: context.style.gutterParagraphStyle
            ]
        )
        if context.layout.showsNoteColumn {
            let hasNote = context.review.notedLines.contains(lineNumber)
            let resolvedOnly = context.review.resolvedOnlyLines.contains(lineNumber)
            gutterLine.append(NSAttributedString(
                string: hasNote ? "● " : "  ",
                attributes: [
                    .font: context.style.font,
                    .foregroundColor: resolvedOnly ? NSColor.tertiaryLabelColor : NSColor.systemOrange,
                    .paragraphStyle: context.style.gutterParagraphStyle
                ]
            ))
        }
        if context.layout.showsCheckboxColumn {
            gutterLine.append(NSAttributedString(
                string: checkboxText(for: lineNumber, line: line),
                attributes: [
                    .font: context.style.font,
                    .foregroundColor: checkboxColor(for: lineNumber, theme: context.style.theme),
                    .paragraphStyle: context.style.gutterParagraphStyle
                ]
            ))
        }
        gutterLine.append(NSAttributedString(
            string: pad(line.oldLineNo.map(String.init) ?? "", toWidth: context.style.maxLineDigits),
            attributes: context.style.gutterAttrs
        ))
        gutterLine.append(NSAttributedString(string: " ", attributes: context.style.gutterAttrs))
        gutterLine.append(NSAttributedString(
            string: pad(line.newLineNo.map(String.init) ?? "", toWidth: context.style.maxLineDigits),
            attributes: context.style.gutterAttrs
        ))
        gutterLine.append(NSAttributedString(string: "\n", attributes: context.style.gutterAttrs))
        return gutterLine
    }
}
