import AppKit
import SwiftUI
import JayJayBindings

/// GitHub Desktop-style two-column diff: left = old, right = new, synced scroll.
struct SideBySideDiffView: View {
    let diff: FileDiff

    var body: some View {
        SideBySideRepresentable(diff: diff)
    }
}

private struct SideBySideRepresentable: NSViewRepresentable {
    let diff: FileDiff

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontScale) private var fontScale

    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeNSView(context: Context) -> NSSplitView {
        let split = NSSplitView()
        split.isVertical = true
        split.dividerStyle = .thin
        split.delegate = context.coordinator

        let left = makeScrollView()
        let right = makeScrollView()
        split.addSubview(left)
        split.addSubview(right)

        context.coordinator.leftScroll = left
        context.coordinator.rightScroll = right
        context.coordinator.startObserving()

        return split
    }

    func updateNSView(_ split: NSSplitView, context: Context) {
        guard let leftScroll = context.coordinator.leftScroll,
              let rightScroll = context.coordinator.rightScroll,
              let leftTV = leftScroll.documentView as? NSTextView,
              let rightTV = rightScroll.documentView as? NSTextView else { return }

        let fontSize = max(10.0, 12.0 * fontScale)
        let font = NSFont.monospacedSystemFont(ofSize: fontSize, weight: .regular)
        let isDark = colorScheme == .dark
        let theme = DiffColors(isDark: isDark)
        let rows = buildRows(from: diff.lines)

        let leftAS = NSMutableAttributedString()
        let rightAS = NSMutableAttributedString()

        for row in rows {
            appendLine(to: leftAS, lineNo: row.oldLineNo, marker: row.oldMarker,
                       spans: row.oldSpans, style: row.oldStyle, font: font, theme: theme)
            appendLine(to: rightAS, lineNo: row.newLineNo, marker: row.newMarker,
                       spans: row.newSpans, style: row.newStyle, font: font, theme: theme)
        }

        if rows.isEmpty {
            let a: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: NSColor.secondaryLabelColor]
            leftAS.append(NSAttributedString(string: "No differences", attributes: a))
        }

        leftTV.textStorage?.setAttributedString(leftAS)
        rightTV.textStorage?.setAttributedString(rightAS)
    }

    private func makeScrollView() -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.autohidesScrollers = true
        scrollView.drawsBackground = false

        let textView = NSTextView()
        textView.isEditable = false
        textView.isSelectable = true
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.autoresizingMask = [.width]
        textView.textContainerInset = NSSize(width: 4, height: 6)
        textView.drawsBackground = false
        textView.textContainer?.widthTracksTextView = true
        textView.textContainer?.lineFragmentPadding = 0

        scrollView.documentView = textView
        return scrollView
    }

    private func appendLine(to str: NSMutableAttributedString, lineNo: String, marker: String,
                            spans: [DiffSpan], style: DiffSpanStyle,
                            font: NSFont, theme: DiffColors) {
        let background = lineBg(style, theme: theme)

        if style == .separator {
            str.append(NSAttributedString(string: " ⋯ \(spans.first?.text ?? "")\n", attributes: [
                .font: font, .foregroundColor: theme.gutterText, .backgroundColor: theme.separatorBg]))
            return
        }

        let padded = lineNo.padding(toLength: 4, withPad: " ", startingAt: 0)
        str.append(NSAttributedString(string: "\(padded) \(marker) ", attributes: [
            .font: font, .foregroundColor: marker == "+" ? theme.addedText : marker == "-" ? theme.removedText : theme.gutterText,
            .backgroundColor: background]))

        if spans.isEmpty {
            str.append(NSAttributedString(string: "\n", attributes: [.font: font, .backgroundColor: background]))
        } else {
            for span in spans {
                let foreground = tokenColor(span.token, fallback: style == .added ? theme.addedText : style == .removed ? theme.removedText : theme.contextText, theme: theme)
                let spanBg: NSColor
                switch span.style {
                case .added: spanBg = theme.addedWordBg
                case .removed: spanBg = theme.removedWordBg
                default: spanBg = background  // line background for unchanged/context
                }
                str.append(NSAttributedString(string: span.text, attributes: [
                    .font: font, .foregroundColor: foreground, .backgroundColor: spanBg]))
            }
            str.append(NSAttributedString(string: "\n", attributes: [.font: font, .backgroundColor: background]))
        }
    }

    private func lineBg(_ s: DiffSpanStyle, theme: DiffColors) -> NSColor {
        switch s {
        case .added: theme.addedBg
        case .removed: theme.removedBg
        case .separator: theme.separatorBg
        default: .clear
        }
    }

    private func tokenColor(_ t: SyntaxToken, fallback: NSColor, theme: DiffColors) -> NSColor {
        switch t {
        case .comment: theme.comment
        case .keyword, .operator: theme.keyword
        case .stringLit: theme.string
        case .number: theme.number
        case .type, .function, .attribute: theme.type
        default: fallback
        }
    }

    final class Coordinator: NSObject, NSSplitViewDelegate {
        func splitView(_ splitView: NSSplitView, constrainMinCoordinate proposedMinimumPosition: CGFloat, ofSubviewAt dividerIndex: Int) -> CGFloat {
            return 100
        }

        func splitView(_ splitView: NSSplitView, resizeSubviewsWithOldSize oldSize: NSSize) {
            let dividerThickness = splitView.dividerThickness
            let halfWidth = (splitView.bounds.width - dividerThickness) / 2
            if splitView.subviews.count >= 2 {
                splitView.subviews[0].frame = NSRect(x: 0, y: 0, width: halfWidth, height: splitView.bounds.height)
                splitView.subviews[1].frame = NSRect(x: halfWidth + dividerThickness, y: 0, width: halfWidth, height: splitView.bounds.height)
            }
        }

        weak var leftScroll: NSScrollView?
        weak var rightScroll: NSScrollView?
        private var syncing = false

        func startObserving() {
            leftScroll?.contentView.postsBoundsChangedNotifications = true
            rightScroll?.contentView.postsBoundsChangedNotifications = true
            NotificationCenter.default.addObserver(self, selector: #selector(leftScrolled), name: NSView.boundsDidChangeNotification, object: leftScroll?.contentView)
            NotificationCenter.default.addObserver(self, selector: #selector(rightScrolled), name: NSView.boundsDidChangeNotification, object: rightScroll?.contentView)
        }

        @objc private func leftScrolled(_ n: Notification) {
            guard !syncing, let o = leftScroll?.contentView.bounds.origin else { return }
            syncing = true; rightScroll?.contentView.scroll(to: o); rightScroll?.reflectScrolledClipView(rightScroll!.contentView); syncing = false
        }

        @objc private func rightScrolled(_ n: Notification) {
            guard !syncing, let o = rightScroll?.contentView.bounds.origin else { return }
            syncing = true; leftScroll?.contentView.scroll(to: o); leftScroll?.reflectScrolledClipView(leftScroll!.contentView); syncing = false
        }

        deinit { NotificationCenter.default.removeObserver(self) }
    }
}

// MARK: - Row model

private struct SBSRow {
    var oldLineNo: String; var oldMarker: String; var oldSpans: [DiffSpan]; var oldStyle: DiffSpanStyle
    var newLineNo: String; var newMarker: String; var newSpans: [DiffSpan]; var newStyle: DiffSpanStyle
}

private func buildRows(from lines: [DiffLine]) -> [SBSRow] {
    var rows: [SBSRow] = []
    var i = 0
    while i < lines.count {
        let line = lines[i]
        switch line.style {
        case .context:
            rows.append(SBSRow(oldLineNo: line.oldLineNo.map(String.init) ?? "", oldMarker: " ", oldSpans: line.spans, oldStyle: .context,
                               newLineNo: line.newLineNo.map(String.init) ?? "", newMarker: " ", newSpans: line.spans, newStyle: .context))
            i += 1
        case .separator:
            rows.append(SBSRow(oldLineNo: "", oldMarker: "", oldSpans: line.spans, oldStyle: .separator,
                               newLineNo: "", newMarker: "", newSpans: line.spans, newStyle: .separator))
            i += 1
        case .removed:
            var removed: [DiffLine] = []
            while i < lines.count && lines[i].style == .removed { removed.append(lines[i]); i += 1 }
            var added: [DiffLine] = []
            while i < lines.count && lines[i].style == .added { added.append(lines[i]); i += 1 }
            for j in 0..<max(removed.count, added.count) {
                let removedLine = j < removed.count ? removed[j] : nil
                let addedLine = j < added.count ? added[j] : nil
                rows.append(SBSRow(
                    oldLineNo: removedLine?.oldLineNo.map(String.init) ?? "", oldMarker: removedLine != nil ? "-" : " ", oldSpans: removedLine?.spans ?? [], oldStyle: removedLine != nil ? .removed : .context,
                    newLineNo: addedLine?.newLineNo.map(String.init) ?? "", newMarker: addedLine != nil ? "+" : " ", newSpans: addedLine?.spans ?? [], newStyle: addedLine != nil ? .added : .context))
            }
        case .added:
            rows.append(SBSRow(oldLineNo: "", oldMarker: " ", oldSpans: [], oldStyle: .context,
                               newLineNo: line.newLineNo.map(String.init) ?? "", newMarker: "+", newSpans: line.spans, newStyle: .added))
            i += 1
        default: i += 1
        }
    }
    return rows
}
