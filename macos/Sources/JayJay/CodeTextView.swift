import AppKit
import SwiftUI

struct SyntaxHighlightedCodeView: NSViewRepresentable {
    let text: String
    let languageHint: String
    let tone: CodePreviewTone

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontScale) private var fontScale

    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = true
        scrollView.borderType = .noBorder
        scrollView.drawsBackground = false

        let textView = NSTextView(frame: .zero)
        textView.isEditable = false
        textView.isSelectable = true
        textView.drawsBackground = false
        textView.isRichText = false
        textView.usesFindBar = true
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = true
        textView.autoresizingMask = [.width]
        textView.minSize = .zero
        textView.maxSize = NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        )
        textView.textContainerInset = NSSize(width: 16, height: 16)
        textView.textContainer?.widthTracksTextView = false
        textView.textContainer?.heightTracksTextView = false
        textView.textContainer?.containerSize = NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        )
        scrollView.documentView = textView
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        guard let textView = scrollView.documentView as? NSTextView else {
            return
        }
        textView.textStorage?.setAttributedString(
            Self.highlighted(
                text: text,
                languageHint: languageHint,
                tone: tone,
                colorScheme: colorScheme,
                fontScale: fontScale
            )
        )
        guard let textContainer = textView.textContainer,
              let layoutManager = textView.layoutManager else {
            return
        }
        layoutManager.ensureLayout(for: textContainer)
        let usedRect = layoutManager.usedRect(for: textContainer)
        let targetSize = NSSize(
            width: max(scrollView.contentSize.width, usedRect.width + 32),
            height: max(scrollView.contentSize.height, usedRect.height + 32)
        )
        textView.frame = NSRect(origin: .zero, size: targetSize)
    }

    private static func highlighted(
        text: String,
        languageHint: String,
        tone: CodePreviewTone,
        colorScheme: ColorScheme,
        fontScale: Double
    ) -> NSAttributedString {
        let font = NSFont.monospacedSystemFont(ofSize: 12 * fontScale, weight: .regular)
        let baseColor = colorScheme == .dark ? NSColor.textColor : NSColor(calibratedWhite: 0.14, alpha: 1)
        let secondary = colorScheme == .dark ? NSColor.systemGray : NSColor.systemGray
        let keyword = NSColor(calibratedRed: 0.16, green: 0.42, blue: 0.92, alpha: 1)
        let typeColor = NSColor(calibratedRed: 0.45, green: 0.29, blue: 0.82, alpha: 1)
        let stringColor = NSColor(calibratedRed: 0.71, green: 0.35, blue: 0.09, alpha: 1)
        let numberColor = NSColor(calibratedRed: 0.12, green: 0.58, blue: 0.49, alpha: 1)

        let paragraph = NSMutableParagraphStyle()
        paragraph.lineHeightMultiple = 1.14

        let attributed = NSMutableAttributedString(
            string: text.isEmpty ? " " : text,
            attributes: [
                .font: font,
                .foregroundColor: baseColor,
                .paragraphStyle: paragraph
            ]
        )

        let fullRange = NSRange(location: 0, length: attributed.length)
        if let background = tone.lineBackgroundColor(colorScheme: colorScheme) {
            attributed.addAttribute(.backgroundColor, value: background, range: fullRange)
        }

        apply(regex: #"(?m)\b(func|let|var|struct|enum|class|protocol|extension|import|return|if|else|for|while|switch|case|default|break|continue|guard|async|await|throws|throw|try|pub|use|fn|impl|match|where|mod|trait|const|static|mut|self|super|in)\b"#, color: keyword, to: attributed)
        apply(regex: #"(?m)\b(Int|String|Bool|Double|Float|Vec|Option|Result|Self|Void|Any|Arc|PathBuf|Date|Color|View)\b"#, color: typeColor, to: attributed)
        apply(regex: #""(?:\\.|[^"\\])*""#, color: stringColor, to: attributed)
        apply(regex: #"(?m)\b\d+(\.\d+)?\b"#, color: numberColor, to: attributed)

        let commentPatterns = commentRegexes(for: languageHint)
        for pattern in commentPatterns {
            apply(regex: pattern, color: secondary, to: attributed)
        }

        return attributed
    }

    private static func commentRegexes(for path: String) -> [String] {
        let ext = URL(fileURLWithPath: path).pathExtension.lowercased()
        switch ext {
        case "py", "sh", "toml", "yml", "yaml", "rb":
            return [#"(?m)#.*$"#]
        case "sql":
            return [#"(?m)--.*$"#]
        default:
            return [#"(?m)//.*$"#, #"(?m)#.*$"#]
        }
    }

    private static func apply(regex pattern: String, color: NSColor, to attributed: NSMutableAttributedString) {
        let range = NSRange(location: 0, length: attributed.length)
        guard let regex = try? NSRegularExpression(pattern: pattern, options: []) else {
            return
        }
        for match in regex.matches(in: attributed.string, options: [], range: range) {
            attributed.addAttribute(.foregroundColor, value: color, range: match.range)
        }
    }
}

enum CodePreviewTone {
    case added
    case removed
    case neutral

    func lineBackgroundColor(colorScheme: ColorScheme) -> NSColor? {
        switch self {
        case .added:
            return NSColor.systemGreen.withAlphaComponent(colorScheme == .dark ? 0.08 : 0.05)
        case .removed:
            return NSColor.systemRed.withAlphaComponent(colorScheme == .dark ? 0.08 : 0.05)
        case .neutral:
            return nil
        }
    }
}
