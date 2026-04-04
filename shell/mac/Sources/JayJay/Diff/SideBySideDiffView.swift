import AppKit
import JayJayCore
import SwiftUI

/// GitHub Desktop-style two-column diff: left = old, right = new, synced scroll.
struct SideBySideDiffView: View {
    let diff: FileDiff

    var body: some View {
        SideBySideRepresentable(diff: diff)
    }
}

struct SideBySideRepresentable: NSViewRepresentable {
    typealias Coordinator = SideBySideCoordinator

    let diff: FileDiff

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontSize) private var fontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    func makeCoordinator() -> SideBySideCoordinator {
        SideBySideCoordinator()
    }

    func makeNSView(context: Context) -> NSSplitView {
        let split = NSSplitView()
        split.isVertical = true
        split.dividerStyle = .thin
        split.delegate = context.coordinator

        let left = makeContainer()
        let right = makeContainer()
        split.addSubview(left)
        split.addSubview(right)

        context.coordinator.leftContainer = left
        context.coordinator.rightContainer = right
        context.coordinator.startObserving()

        return split
    }

    func updateNSView(_ split: NSSplitView, context: Context) {
        guard let leftContainer = context.coordinator.leftContainer,
              let rightContainer = context.coordinator.rightContainer,
              let leftTV = leftContainer.textView as NSTextView?,
              let rightTV = rightContainer.textView as NSTextView?,
              let leftGutterTV = leftContainer.gutterTextView as DiffGutterTextView?,
              let rightGutterTV = rightContainer.gutterTextView as DiffGutterTextView?,
              let leftLayout = leftTV.layoutManager as? DiffLayoutManager,
              let rightLayout = rightTV.layoutManager as? DiffLayoutManager,
              let leftGutterLayout = leftGutterTV.layoutManager as? DiffLayoutManager,
              let rightGutterLayout = rightGutterTV.layoutManager as? DiffLayoutManager
        else { return }

        let font = fontFamily.nsFont(size: fontSize)
        let theme = DiffColors(isDark: colorScheme == .dark)
        let rows = buildSideBySideRows(lines: diff.lines)

        let leftText = NSMutableAttributedString()
        let rightText = NSMutableAttributedString()
        let leftGutter = NSMutableAttributedString()
        let rightGutter = NSMutableAttributedString()
        var leftEntries: [DiffGutterTextView.Entry] = []
        var rightEntries: [DiffGutterTextView.Entry] = []
        var leftWidth: CGFloat = 0
        var rightWidth: CGFloat = 0
        var leftColors: [NSColor] = []
        var rightColors: [NSColor] = []

        let gutterParagraphStyle = NSMutableParagraphStyle()
        gutterParagraphStyle.alignment = .right
        let gutterAttrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: theme.gutterText,
            .paragraphStyle: gutterParagraphStyle
        ]
        let markerWidth = ("+" as NSString).size(withAttributes: [.font: font]).width
        let gutterGap: CGFloat = 10
        let trailingPadding: CGFloat = 10

        for row in rows {
            appendTextLine(
                to: leftText,
                spans: row.oldSpans,
                style: row.oldStyle,
                font: font,
                theme: theme,
                bgColors: &leftColors
            )
            appendTextLine(
                to: rightText,
                spans: row.newSpans,
                style: row.newStyle,
                font: font,
                theme: theme,
                bgColors: &rightColors
            )
            appendGutterLine(
                to: leftGutter,
                entries: &leftEntries,
                lineNo: row.oldLineNo,
                marker: row.oldMarker,
                style: row.oldStyle,
                attrs: gutterAttrs,
                font: font,
                markerWidth: markerWidth,
                inset: leftGutterTV.textContainerInset.width,
                gap: gutterGap,
                trailingPadding: trailingPadding,
                width: &leftWidth,
                theme: theme
            )
            appendGutterLine(
                to: rightGutter,
                entries: &rightEntries,
                lineNo: row.newLineNo,
                marker: row.newMarker,
                style: row.newStyle,
                attrs: gutterAttrs,
                font: font,
                markerWidth: markerWidth,
                inset: rightGutterTV.textContainerInset.width,
                gap: gutterGap,
                trailingPadding: trailingPadding,
                width: &rightWidth,
                theme: theme
            )
        }

        if rows.isEmpty {
            let attrs: [NSAttributedString.Key: Any] = [
                .font: font,
                .foregroundColor: NSColor.secondaryLabelColor
            ]
            leftText.append(NSAttributedString(string: "No differences", attributes: attrs))
            leftGutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
            rightGutter.append(NSAttributedString(string: "\n", attributes: gutterAttrs))
        }

        leftLayout.lineBgColors = leftColors
        rightLayout.lineBgColors = rightColors
        leftGutterLayout.lineBgColors = leftColors
        rightGutterLayout.lineBgColors = rightColors
        leftTV.textStorage?.setAttributedString(leftText)
        rightTV.textStorage?.setAttributedString(rightText)
        leftGutterTV.textStorage?.setAttributedString(leftGutter)
        rightGutterTV.textStorage?.setAttributedString(rightGutter)
        leftGutterTV.entries = leftEntries
        rightGutterTV.entries = rightEntries
        leftContainer.updateGutterWidth(max(52, leftWidth))
        rightContainer.updateGutterWidth(max(52, rightWidth))
    }
}
