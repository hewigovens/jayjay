import AppKit
import JayJayCore
import JayJayDiffUI

extension CodeTextView {
    @MainActor
    final class Coordinator: NSObject, NSTextViewDelegate {
        private var parent: CodeTextView
        private var scrollAnchor: MergeTextScrollAnchor?
        private var highlightTask: Task<Void, Never>?
        private var revision: UInt64 = 0
        private var isApplyingExternalText = false
        private var didApplyPreparedHighlighting = false
        private var didRevealInitialLine = false

        init(parent: CodeTextView) {
            self.parent = parent
        }

        deinit {
            highlightTask?.cancel()
        }

        func detachScroll() {
            if let pane = parent.mergePane, let scrollAnchor {
                parent.mergeScroll?.unregister(pane, anchor: scrollAnchor)
            }
            scrollAnchor = nil
        }

        func update(parent: CodeTextView, textView: NSTextView) {
            if self.parent.mergePane != parent.mergePane || self.parent.mergeScroll !== parent.mergeScroll {
                detachScroll()
            }
            let pathChanged = self.parent.path != parent.path
            let fontChanged = self.parent.typography != parent.typography
            let appearanceChanged = self.parent.highlightColors != parent.highlightColors
            let preparedHighlightsChanged = self.parent.preparedHighlightedLines != parent.preparedHighlightedLines
                || self.parent.preparedLineStyles != parent.preparedLineStyles
            self.parent = parent
            if fontChanged {
                parent.applyTypography(to: textView)
                (textView.enclosingScrollView?.verticalRulerView as? CodeLineNumberRuler)?.updateText()
            }
            textView.isEditable = parent.isEditable
            textView.setAccessibilityIdentifier(parent.accessibilityIdentifier)

            let textChanged = textView.string != parent.text
            if textChanged {
                isApplyingExternalText = true
                let selection = textView.selectedRange()
                textView.string = parent.text
                let textLength = (parent.text as NSString).length
                let location = min(selection.location, textLength)
                textView.setSelectedRange(NSRange(
                    location: location,
                    length: min(selection.length, textLength - location)
                ))
                isApplyingExternalText = false
                (textView.enclosingScrollView?.verticalRulerView as? CodeLineNumberRuler)?.updateText()
            }
            scrollAnchor?.updateText()
            let canApplyPreparedHighlighting = parent.preparedText == parent.text
                && parent.preparedHighlightedLines != nil
            if canApplyPreparedHighlighting,
               textChanged || pathChanged || appearanceChanged || fontChanged || preparedHighlightsChanged || !didApplyPreparedHighlighting,
               let highlightedLines = parent.preparedHighlightedLines,
               let storage = textView.textStorage
            {
                highlightTask?.cancel()
                storage.beginEditing()
                CodeHighlighting(font: parent.typography.editorFont, colors: parent.highlightColors).apply(
                    to: storage,
                    text: parent.text,
                    highlightedLines: highlightedLines,
                    lineStyles: parent.preparedLineStyles ?? []
                )
                storage.endEditing()
                updateTypingAttributes(of: textView, colors: parent.highlightColors)
                didApplyPreparedHighlighting = true
            } else if textChanged || pathChanged || appearanceChanged || fontChanged || preparedHighlightsChanged || highlightTask == nil {
                didApplyPreparedHighlighting = false
                scheduleHighlight(for: textView, debounce: false)
            }
            revealInitialLine(in: textView)
            if scrollAnchor == nil, let pane = parent.mergePane, let scroll = parent.mergeScroll {
                let anchor = MergeTextScrollAnchor(textView: textView) { [weak scroll] in
                    scroll?.didScroll(pane)
                }
                scrollAnchor = anchor
                scroll.register(pane, anchor: anchor)
            }
        }

        private func revealInitialLine(in textView: NSTextView) {
            guard !didRevealInitialLine, let line = parent.initialLine, !textView.string.isEmpty else { return }
            didRevealInitialLine = true
            let location = (textView.string as NSString).location(ofLine: Int(line))
            textView.setSelectedRange(NSRange(location: location, length: 0))
            // The scroll view has no size until the sheet lays out.
            DispatchQueue.main.async { [weak textView] in
                guard let textView,
                      let layoutManager = textView.layoutManager,
                      let container = textView.textContainer,
                      let clipView = textView.enclosingScrollView?.contentView
                else { return }
                let glyphs = layoutManager.glyphRange(forCharacterRange: NSRange(location: location, length: 0), actualCharacterRange: nil)
                let lineRect = layoutManager.boundingRect(forGlyphRange: glyphs, in: container)
                let y = lineRect.midY + textView.textContainerOrigin.y - clipView.bounds.height / 2
                let maxY = max(textView.frame.height - clipView.bounds.height, 0)
                clipView.scroll(to: NSPoint(x: 0, y: min(max(y, 0), maxY)))
                textView.enclosingScrollView?.reflectScrolledClipView(clipView)
                textView.window?.makeFirstResponder(textView)
            }
        }

        func textDidChange(_ notification: Notification) {
            guard !isApplyingExternalText, let textView = notification.object as? NSTextView else { return }
            scrollAnchor?.updateText()
            (textView.enclosingScrollView?.verticalRulerView as? CodeLineNumberRuler)?.updateText()
            parent.mergeScroll?.invalidateResult()
            parent.text = textView.string
            parent.onTextChanged()
            didApplyPreparedHighlighting = false
            scheduleHighlight(for: textView, debounce: true)
        }

        private func scheduleHighlight(for textView: NSTextView, debounce: Bool) {
            revision &+= 1
            let requestRevision = revision
            let path = parent.path
            let text = textView.string
            let colors = parent.highlightColors
            highlightTask?.cancel()
            highlightTask = Task { @MainActor [weak self, weak textView] in
                if debounce {
                    try? await Task.sleep(nanoseconds: 120_000_000)
                }
                guard !Task.isCancelled else { return }
                let lines = await Task.detached(priority: .userInitiated) {
                    highlightFileLines(path: path, content: text)
                }.value
                guard
                    !Task.isCancelled,
                    let self,
                    revision == requestRevision,
                    let textView,
                    textView.string == text,
                    !textView.hasMarkedText(),
                    let storage = textView.textStorage
                else { return }
                storage.beginEditing()
                CodeHighlighting(font: parent.typography.editorFont, colors: colors).apply(
                    to: storage,
                    text: text,
                    highlightedLines: lines
                )
                storage.endEditing()
                updateTypingAttributes(of: textView, colors: colors)
            }
        }

        private func updateTypingAttributes(of textView: NSTextView, colors: DiffColors) {
            textView.typingAttributes = CodeHighlighting(font: parent.typography.editorFont, colors: colors).baseAttributes
        }
    }
}
