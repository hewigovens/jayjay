import AppKit
import JayJayCore
import SwiftUI

struct CodeTextView: NSViewRepresentable {
    enum Presentation {
        case plain
        case editorPane
    }

    let path: String
    @Binding var text: String
    let isEditable: Bool
    let wrapsLines: Bool
    let presentation: Presentation
    let accessibilityIdentifier: String?
    let onTextChanged: () -> Void
    let preparedText: String?
    let preparedHighlightedLines: [[DiffSpan]]?
    let preparedLineStyles: [DiffSpanStyle]?

    @Environment(\.colorScheme) var colorScheme

    static let editorFont = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)

    init(
        path: String,
        text: Binding<String>,
        isEditable: Bool,
        wrapsLines: Bool = false,
        presentation: Presentation = .plain,
        accessibilityIdentifier: String? = nil,
        onTextChanged: @escaping () -> Void = {},
        preparedText: String? = nil,
        preparedHighlightedLines: [[DiffSpan]]? = nil,
        preparedLineStyles: [DiffSpanStyle]? = nil
    ) {
        self.path = path
        _text = text
        self.isEditable = isEditable
        self.wrapsLines = wrapsLines
        self.presentation = presentation
        self.accessibilityIdentifier = accessibilityIdentifier
        self.onTextChanged = onTextChanged
        self.preparedText = preparedText
        self.preparedHighlightedLines = preparedHighlightedLines
        self.preparedLineStyles = preparedLineStyles
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.autohidesScrollers = true
        Self.configurePresentation(presentation, scrollView: scrollView)

        let textContainer = NSTextContainer(containerSize: NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        ))
        textContainer.lineFragmentPadding = 0

        let layoutManager = NSLayoutManager()
        layoutManager.addTextContainer(textContainer)
        let storage = NSTextStorage()
        storage.addLayoutManager(layoutManager)

        let textView = NSTextView(frame: scrollView.bounds, textContainer: textContainer)
        textView.delegate = context.coordinator
        textView.isEditable = isEditable
        textView.isSelectable = true
        textView.isRichText = false
        textView.allowsUndo = true
        textView.drawsBackground = false
        textView.isVerticallyResizable = true
        textView.autoresizingMask = [.width]
        textView.textContainerInset = NSSize(width: 10, height: 10)
        textView.minSize = .zero
        textView.maxSize = NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        )
        textView.usesFindBar = true
        textView.isIncrementalSearchingEnabled = true
        textView.isContinuousSpellCheckingEnabled = false
        textView.isGrammarCheckingEnabled = false
        textView.isAutomaticSpellingCorrectionEnabled = false
        textView.isAutomaticQuoteSubstitutionEnabled = false
        textView.isAutomaticDashSubstitutionEnabled = false
        textView.isAutomaticTextReplacementEnabled = false
        scrollView.documentView = textView
        Self.configureLineWrapping(wrapsLines, textView: textView, scrollView: scrollView)
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        guard let textView = scrollView.documentView as? NSTextView else { return }
        Self.configurePresentation(presentation, scrollView: scrollView)
        Self.configureLineWrapping(wrapsLines, textView: textView, scrollView: scrollView)
        context.coordinator.update(parent: self, textView: textView)
    }

    private static func configurePresentation(_ presentation: Presentation, scrollView: NSScrollView) {
        scrollView.drawsBackground = presentation == .editorPane
        guard presentation == .editorPane else {
            scrollView.backgroundColor = .clear
            return
        }
        let isDark = scrollView.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        scrollView.backgroundColor = NSColor.labelColor.withAlphaComponent(isDark ? 0.07 : 0.04)
    }

    private static func configureLineWrapping(
        _ wrapsLines: Bool,
        textView: NSTextView,
        scrollView: NSScrollView
    ) {
        scrollView.hasHorizontalScroller = !wrapsLines
        textView.isHorizontallyResizable = !wrapsLines
        textView.textContainer?.widthTracksTextView = wrapsLines
        textView.textContainer?.containerSize = NSSize(
            width: wrapsLines ? scrollView.contentSize.width : CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        )
    }
}
