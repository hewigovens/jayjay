import AppKit
import JayJayCore
import SwiftUI

/// GitHub Desktop-style two-column diff: left = old, right = new, synced scroll.
public struct SideBySideDiffView: View {
    public let diff: FileDiff

    public init(diff: FileDiff) {
        self.diff = diff
    }

    public var body: some View {
        SideBySideRepresentable(diff: diff)
    }
}

public struct SideBySideRepresentable: NSViewRepresentable {
    public typealias Coordinator = SideBySideCoordinator

    public let diff: FileDiff

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.diffFontSize) private var fontSize
    @Environment(\.diffFontFamily) private var fontFamily

    public init(diff: FileDiff) {
        self.diff = diff
    }

    public func makeCoordinator() -> SideBySideCoordinator {
        SideBySideCoordinator()
    }

    public func makeNSView(context: Context) -> NSSplitView {
        let split = NSSplitView()
        split.isVertical = true
        split.dividerStyle = .thin
        split.delegate = context.coordinator

        let left = makeContainer()
        let right = makeContainer()
        // Per-side wrapping desyncs the two panes' visual rows from each other and
        // from the gutters. Disable until SBS gains "wrap to tallest side" alignment.
        left.wrapsText = false
        right.wrapsText = false
        split.addSubview(left)
        split.addSubview(right)

        context.coordinator.leftContainer = left
        context.coordinator.rightContainer = right
        context.coordinator.startObserving()

        return split
    }

    public func updateNSView(_ split: NSSplitView, context: Context) {
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

        let font = NSFont(name: fontFamily, size: fontSize) ?? .monospacedSystemFont(ofSize: fontSize, weight: .regular)
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
                style: row.oldStyle,
                attrs: gutterAttrs,
                inset: leftGutterTV.textContainerInset.width,
                trailingPadding: trailingPadding,
                width: &leftWidth
            )
            appendGutterLine(
                to: rightGutter,
                entries: &rightEntries,
                lineNo: row.newLineNo,
                style: row.newStyle,
                attrs: gutterAttrs,
                inset: rightGutterTV.textContainerInset.width,
                trailingPadding: trailingPadding,
                width: &rightWidth
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
