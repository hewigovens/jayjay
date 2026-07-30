import AppKit
import JayJayCore

struct PaneAccumulator {
    let pane: PaneViews
    let text = NSMutableAttributedString()
    let gutter = NSMutableAttributedString()
    var entries: [DiffGutterTextView.Entry] = []
    var width: CGFloat = 0
    var colors: [NSColor] = []
    var stripes: [NSColor] = []
    var viewportLineLocations: [DiffViewportLineLocation] = []

    mutating func append(
        _ side: RowSide,
        contextRegion: ContextRegion?,
        viewportIdentity: DiffViewportLineIdentity?,
        enablesContextExpansion: Bool,
        font: NSFont,
        theme: DiffColors,
        gutterAttrs: [NSAttributedString.Key: Any],
        trailingPadding: CGFloat
    ) {
        let textStart = text.length
        appendTextLine(
            to: text,
            spans: side.spans,
            style: side.style,
            conflictKind: side.conflictKind,
            contextRegion: contextRegion,
            enablesContextExpansion: enablesContextExpansion,
            font: font,
            theme: theme,
            bgColors: &colors
        )
        if let viewportIdentity {
            viewportLineLocations.append(DiffViewportLineLocation(
                identity: viewportIdentity,
                characterRange: NSRange(
                    location: textStart,
                    length: text.length - textStart
                )
            ))
        }
        stripes.append(conflictStripe(conflictKind: side.conflictKind, theme: theme))
        appendGutterLine(
            to: gutter,
            entries: &entries,
            lineNo: side.lineNo,
            style: side.style,
            attrs: gutterAttrs,
            inset: pane.gutterTextView.textContainerInset.width,
            trailingPadding: trailingPadding,
            width: &width
        )
    }

    func commit(
        restoring anchor: DiffViewportAnchor?,
        revealFeedback: DiffContextRevealFeedback?,
        reduceMotion: Bool
    ) {
        pane.textLayout.lineBgColors = colors
        pane.textLayout.lineStripeColors = stripes
        pane.textLayout.lineStripeX = 0
        pane.textLayout.lineStripeWidth = 3
        pane.gutterLayout.lineBgColors = colors
        pane.gutterLayout.lineStripeColors = stripes
        pane.gutterLayout.lineStripeX = 0
        pane.gutterLayout.lineStripeWidth = 3
        pane.textView.textStorage?.setAttributedString(text)
        pane.gutterTextView.textStorage?.setAttributedString(gutter)
        pane.gutterTextView.entries = entries
        pane.container.updateGutterWidth(max(DiffGutterMetrics.minimumUnifiedWidth, width))
        pane.container.setViewportLineLocations(
            viewportLineLocations,
            restoring: anchor
        )
        pane.container.scheduleRevealFeedback(
            revealFeedback,
            reduceMotion: reduceMotion
        )
    }
}
