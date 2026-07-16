import AppKit
import SwiftUI

struct SelectableTextView: NSViewRepresentable {
    struct Line {
        let text: String
        let color: NSColor
    }

    let lines: [Line]
    let lineHeight: CGFloat

    func makeNSView(context _: Context) -> NSTextView {
        let view = NSTextView()
        view.isEditable = false
        view.isSelectable = true
        view.drawsBackground = false
        view.textContainerInset = .zero
        view.textContainer?.lineFragmentPadding = 0
        view.textContainer?.widthTracksTextView = false
        view.textContainer?.containerSize = NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        )
        view.clipsToBounds = true
        view.font = .monospacedSystemFont(ofSize: 11, weight: .regular)
        view.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        return view
    }

    func updateNSView(_ view: NSTextView, context _: Context) {
        let text = attributedText()
        guard view.attributedString() != text else { return }
        view.textStorage?.setAttributedString(text)
    }

    private func attributedText() -> NSAttributedString {
        let result = NSMutableAttributedString()
        let paragraph = NSMutableParagraphStyle()
        paragraph.minimumLineHeight = lineHeight
        paragraph.maximumLineHeight = lineHeight
        let font = NSFont.monospacedSystemFont(ofSize: 11, weight: .regular)
        for (index, line) in lines.enumerated() {
            let attributes: [NSAttributedString.Key: Any] = [
                .font: font,
                .foregroundColor: line.color,
                .paragraphStyle: paragraph
            ]
            result.append(NSAttributedString(string: line.text, attributes: attributes))
            if index < lines.count - 1 {
                result.append(NSAttributedString(string: "\n", attributes: attributes))
            }
        }
        return result
    }
}
