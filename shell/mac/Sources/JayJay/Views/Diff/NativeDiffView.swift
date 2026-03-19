import AppKit
import SwiftUI
import JayJayBindings

struct NativeDiffView: NSViewRepresentable {
    let diff: FileDiff

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontScale) private var fontScale

    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = true
        scrollView.autohidesScrollers = true
        scrollView.drawsBackground = false

        let textView = NSTextView()
        textView.isEditable = false
        textView.isSelectable = true
        textView.autoresizingMask = [.width]
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.textContainerInset = NSSize(width: 8, height: 8)
        textView.drawsBackground = false
        textView.textContainer?.widthTracksTextView = true
        textView.textContainer?.lineFragmentPadding = 0

        scrollView.documentView = textView
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        guard let textView = scrollView.documentView as? NSTextView else { return }

        let fontSize = max(10.0, 12.0 * fontScale)
        let font = NSFont.monospacedSystemFont(ofSize: fontSize, weight: .regular)
        let boldFont = NSFont.monospacedSystemFont(ofSize: fontSize, weight: .medium)
        let isDark = colorScheme == .dark
        let theme = DiffTheme(isDark: isDark)

        let result = NSMutableAttributedString()

        let lineNumberWidth = maxLineNumberWidth(font: font)

        for line in diff.lines {
            let lineStr = NSMutableAttributedString()

            // Line number (prefer new, fall back to old for removed lines)
            let lineNo = (line.newLineNo ?? line.oldLineNo).map { String(format: "%4d", $0) } ?? "    "
            let gutter = "\(lineNo) "

            lineStr.append(NSAttributedString(string: gutter, attributes: [
                .font: font,
                .foregroundColor: theme.gutterText,
            ]))

            // Diff marker
            // Separator line (collapsed context)
            if line.style == .separator {
                let text = "  ⋯ \(line.spans.first?.text ?? "")  "
                let sepAttrs: [NSAttributedString.Key: Any] = [
                    .font: font,
                    .foregroundColor: theme.gutterText,
                    .backgroundColor: isDark
                        ? NSColor(white: 0.16, alpha: 1)
                        : NSColor(white: 0.94, alpha: 1),
                ]
                result.append(NSAttributedString(string: text + "\n", attributes: sepAttrs))
                continue
            }

            let marker: String
            let lineBackground: NSColor
            switch line.style {
            case .added:
                marker = "+ "
                lineBackground = theme.addedBackground
            case .removed:
                marker = "- "
                lineBackground = theme.removedBackground
            case .context, .unchanged, .separator:
                marker = "  "
                lineBackground = .clear
            }
            lineStr.append(NSAttributedString(string: marker, attributes: [
                .font: boldFont,
                .foregroundColor: markerColor(line.style, theme: theme),
            ]))

            // Content spans
            for span in line.spans {
                let fg = foregroundColor(span: span, lineStyle: line.style, theme: theme)
                let bg = spanBackground(span: span, theme: theme)

                var attrs: [NSAttributedString.Key: Any] = [
                    .font: font,
                    .foregroundColor: fg,
                ]
                if bg != .clear {
                    attrs[.backgroundColor] = bg
                }
                lineStr.append(NSAttributedString(string: span.text, attributes: attrs))
            }

            // Apply line-level background
            if lineBackground != .clear {
                lineStr.addAttribute(.backgroundColor, value: lineBackground,
                                     range: NSRange(location: 0, length: lineStr.length))
            }

            lineStr.append(NSAttributedString(string: "\n", attributes: [.font: font]))
            result.append(lineStr)
        }

        if diff.lines.isEmpty {
            result.append(NSAttributedString(string: "No differences",
                                             attributes: [.font: font, .foregroundColor: NSColor.secondaryLabelColor]))
        }

        textView.textStorage?.setAttributedString(result)
    }

    private func maxLineNumberWidth(font: NSFont) -> CGFloat {
        let sample = NSAttributedString(string: "9999 9999  ", attributes: [.font: font])
        return sample.size().width
    }

    private func markerColor(_ style: DiffSpanStyle, theme: DiffTheme) -> NSColor {
        switch style {
        case .added: theme.addedText
        case .removed: theme.removedText
        default: theme.contextText
        }
    }

    private func foregroundColor(span: NativeDiffSpan, lineStyle: DiffSpanStyle, theme: DiffTheme) -> NSColor {
        // Syntax token coloring
        switch span.token {
        case .comment: return theme.comment
        case .keyword: return theme.keyword
        case .stringLit: return theme.string
        case .number: return theme.number
        case .type: return theme.type
        case .function: return theme.function
        case .attribute: return theme.attribute
        case .operator: return theme.operator
        case .punctuation: return theme.punctuation
        default: break
        }

        // Diff-style coloring for plain tokens
        switch lineStyle {
        case .added: return theme.addedText
        case .removed: return theme.removedText
        default: return theme.contextText
        }
    }

    private func spanBackground(span: NativeDiffSpan, theme: DiffTheme) -> NSColor {
        switch span.style {
        case .added: return theme.addedWordBackground
        case .removed: return theme.removedWordBackground
        default: return .clear
        }
    }
}

private struct DiffTheme {
    let isDark: Bool

    var gutterText: NSColor {
        isDark ? NSColor(white: 0.45, alpha: 1) : NSColor(white: 0.65, alpha: 1)
    }
    var contextText: NSColor {
        isDark ? NSColor(white: 0.85, alpha: 1) : NSColor(white: 0.15, alpha: 1)
    }

    // Added
    var addedText: NSColor {
        isDark ? NSColor(red: 0.47, green: 0.91, blue: 0.53, alpha: 1) : NSColor(red: 0.08, green: 0.47, blue: 0.17, alpha: 1)
    }
    var addedBackground: NSColor {
        isDark ? NSColor(red: 0.07, green: 0.15, blue: 0.12, alpha: 1) : NSColor(red: 0.85, green: 0.98, blue: 0.88, alpha: 1)
    }
    var addedWordBackground: NSColor {
        isDark ? NSColor(red: 0.15, green: 0.42, blue: 0.22, alpha: 0.5) : NSColor(red: 0.67, green: 0.93, blue: 0.73, alpha: 0.6)
    }

    // Removed
    var removedText: NSColor {
        isDark ? NSColor(red: 1, green: 0.48, blue: 0.45, alpha: 1) : NSColor(red: 0.82, green: 0.17, blue: 0.14, alpha: 1)
    }
    var removedBackground: NSColor {
        isDark ? NSColor(red: 0.18, green: 0.08, blue: 0.08, alpha: 1) : NSColor(red: 1, green: 0.93, blue: 0.94, alpha: 1)
    }
    var removedWordBackground: NSColor {
        isDark ? NSColor(red: 0.52, green: 0.15, blue: 0.15, alpha: 0.5) : NSColor(red: 1, green: 0.78, blue: 0.78, alpha: 0.6)
    }

    // Syntax tokens (GitHub-inspired)
    var keyword: NSColor {
        isDark ? NSColor(red: 1, green: 0.48, blue: 0.45, alpha: 1) : NSColor(red: 0.84, green: 0.23, blue: 0.29, alpha: 1)
    }
    var string: NSColor {
        isDark ? NSColor(red: 0.65, green: 0.84, blue: 1, alpha: 1) : NSColor(red: 0.01, green: 0.18, blue: 0.38, alpha: 1)
    }
    var comment: NSColor {
        isDark ? NSColor(red: 0.55, green: 0.58, blue: 0.63, alpha: 1) : NSColor(red: 0.42, green: 0.45, blue: 0.49, alpha: 1)
    }
    var number: NSColor {
        isDark ? NSColor(red: 0.47, green: 0.75, blue: 1, alpha: 1) : NSColor(red: 0, green: 0.36, blue: 0.77, alpha: 1)
    }
    var type: NSColor {
        isDark ? NSColor(red: 0.82, green: 0.66, blue: 1, alpha: 1) : NSColor(red: 0.44, green: 0.26, blue: 0.76, alpha: 1)
    }
    var function: NSColor {
        isDark ? NSColor(red: 0.82, green: 0.66, blue: 1, alpha: 1) : NSColor(red: 0.44, green: 0.26, blue: 0.76, alpha: 1)
    }
    var attribute: NSColor {
        isDark ? NSColor(red: 0.82, green: 0.66, blue: 1, alpha: 1) : NSColor(red: 0.44, green: 0.26, blue: 0.76, alpha: 1)
    }
    var `operator`: NSColor { keyword }
    var punctuation: NSColor { contextText }
}
