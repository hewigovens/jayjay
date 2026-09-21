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
            let fontChanged = self.parent.fontSize != parent.fontSize || self.parent.fontFamily != parent.fontFamily
            let appearanceChanged = self.parent.colorScheme != parent.colorScheme
            let preparedHighlightsChanged = self.parent.preparedHighlightedLines != parent.preparedHighlightedLines
                || self.parent.preparedLineStyles != parent.preparedLineStyles
            self.parent = parent
            if fontChanged {
                textView.font = parent.editorFont
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
                CodeHighlighting(font: parent.editorFont, isDark: parent.colorScheme == .dark).apply(
                    to: storage,
                    text: parent.text,
                    highlightedLines: highlightedLines,
                    lineStyles: parent.preparedLineStyles ?? []
                )
                storage.endEditing()
                updateTypingAttributes(of: textView, isDark: parent.colorScheme == .dark)
                didApplyPreparedHighlighting = true
            } else if textChanged || pathChanged || appearanceChanged || fontChanged || preparedHighlightsChanged || highlightTask == nil {
                didApplyPreparedHighlighting = false
                scheduleHighlight(for: textView, debounce: false)
            }
            if scrollAnchor == nil, let pane = parent.mergePane, let scroll = parent.mergeScroll {
                let anchor = MergeTextScrollAnchor(textView: textView) { [weak scroll] in
                    scroll?.didScroll(pane)
                }
                scrollAnchor = anchor
                scroll.register(pane, anchor: anchor)
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
            let isDark = parent.colorScheme == .dark
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
                CodeHighlighting(font: parent.editorFont, isDark: isDark).apply(
                    to: storage,
                    text: text,
                    highlightedLines: lines
                )
                storage.endEditing()
                updateTypingAttributes(of: textView, isDark: isDark)
            }
        }

        private func updateTypingAttributes(of textView: NSTextView, isDark: Bool) {
            textView.typingAttributes = CodeHighlighting(font: parent.editorFont, isDark: isDark).baseAttributes
        }
    }
}
