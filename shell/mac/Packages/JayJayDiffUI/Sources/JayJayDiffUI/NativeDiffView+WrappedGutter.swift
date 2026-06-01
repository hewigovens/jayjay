import AppKit
import JayJayCore

struct NativeDiffGutterRenderContext {
    let lines: [DiffLine]
    let visualLineCounts: [Int]
    let font: NSFont
    let theme: DiffColors
    let gutterAttrs: [NSAttributedString.Key: Any]
    let gutterParagraphStyle: NSMutableParagraphStyle
    let maxLineDigits: Int
    let showsReviewCheckboxes: Bool
    let showsCheckboxColumn: Bool
    let firstLineOfGroup: [Int: UInt32]
    let groupIndexAtLineNumber: [Int: UInt32]
    let reviewActions: (any DiffGutterReviewActions)?
    let groupStripeWidth: CGFloat
    let gutterHorizontalInset: CGFloat
    let gutterTrailingPadding: CGFloat
    let currentSelectedLineRange: ClosedRange<Int>?
}

extension NativeDiffGutterRenderContext {
    var blankGutterLine: NSAttributedString {
        let blankNumber = String(repeating: " ", count: maxLineDigits)
        let leftColumn = showsReviewCheckboxes ? "   " : "  "
        let checkboxColumn = showsCheckboxColumn ? "  " : ""
        return NSAttributedString(
            string: "\(leftColumn)\(checkboxColumn)\(blankNumber) \(blankNumber)\n",
            attributes: gutterAttrs
        )
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

        for (index, line) in context.lines.enumerated() {
            let lineNumber = index + 1
            let visualRows = index < context.visualLineCounts.count
                ? max(1, context.visualLineCounts[index])
                : 1
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

        if context.lines.isEmpty {
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
        if context.showsReviewCheckboxes,
           let groupIdx = context.groupIndexAtLineNumber[lineNumber]
        {
            return context.reviewActions?.isHunkReviewed(groupIndex: groupIdx) == true
                ? NSColor.controlAccentColor
                : NSColor.selectedTextBackgroundColor
        }
        return groupStripeColor(
            for: line,
            groupRange: expandedHunkRange(containing: lineNumber ... lineNumber),
            theme: context.theme
        )
    }

    private func firstVisualGutterLine(
        for line: DiffLine,
        lineNumber: Int,
        context: NativeDiffGutterRenderContext
    ) -> NSMutableAttributedString {
        let isFirstOfReviewedGroup = context.showsReviewCheckboxes
            && context.firstLineOfGroup[lineNumber]
            .map { context.reviewActions?.isHunkReviewed(groupIndex: $0) == true } == true
        let leftColumnString: String = if context.showsReviewCheckboxes {
            isFirstOfReviewedGroup ? " ✓ " : "   "
        } else {
            "  "
        }
        let gutterLine = NSMutableAttributedString(
            string: leftColumnString,
            attributes: [
                .font: context.font,
                .foregroundColor: isFirstOfReviewedGroup
                    ? NSColor.controlAccentColor
                    : context.theme.gutterText,
                .paragraphStyle: context.gutterParagraphStyle
            ]
        )
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
