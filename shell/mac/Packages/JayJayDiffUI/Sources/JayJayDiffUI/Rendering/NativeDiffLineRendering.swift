import AppKit
import JayJayCore

struct NativeDiffLineRenderContext {
    let font: NSFont
    let theme: DiffColors
    let enablesContextExpansion: Bool
}

func appendNativeDiffLine(
    _ line: DiffLine,
    to result: NSMutableAttributedString,
    context: NativeDiffLineRenderContext,
    bgColors: inout [NSColor],
    viewportLineLocations: inout [DiffViewportLineLocation]
) {
    let lineStart = result.length
    if line.style == .separator {
        result.append(DiffContextExpansionLink.separatorString(
            text: line.spans.first?.text ?? "",
            region: line.contextRegion,
            enablesExpansion: context.enablesContextExpansion,
            font: context.font,
            foregroundColor: context.theme.gutterText
        ))
        bgColors.append(context.theme.separatorBg)
        if let identity = DiffViewportLineIdentity.unified(line) {
            viewportLineLocations.append(DiffViewportLineLocation(
                identity: identity,
                characterRange: NSRange(
                    location: lineStart,
                    length: result.length - lineStart
                )
            ))
        }
        return
    }

    if let label = conflictLabel(for: line) {
        result.append(NSAttributedString(
            string: conflictDisplayLine(label: label, kind: line.conflictKind),
            attributes: conflictLineAttributes(
                kind: line.conflictKind,
                font: context.font,
                theme: context.theme
            )
        ))
    } else {
        for span in line.spans {
            let attrs = diffSpanAttributes(
                for: span,
                lineStyle: line.style,
                conflictKind: line.conflictKind,
                font: context.font,
                theme: context.theme
            )
            result.append(NSAttributedString(string: span.text, attributes: attrs))
        }
        if line.spans.isEmpty {
            result.append(NSAttributedString(string: " ", attributes: [.font: context.font]))
        }
    }
    if line.noEofNewline {
        let dim: [NSAttributedString.Key: Any] = [
            .font: context.font,
            .foregroundColor: context.theme.gutterText
        ]
        result.append(NSAttributedString(string: "  ⊘", attributes: dim))
        var arrowAttrs = dim
        // ↵ sits lower than ⊘ in most monospace fonts; nudge it up to match visual center.
        arrowAttrs[.baselineOffset] = 1.5
        result.append(NSAttributedString(string: "↵", attributes: arrowAttrs))
        result.append(NSAttributedString(string: "  no newline at EOF", attributes: dim))
    }
    result.append(NSAttributedString(string: "\n", attributes: [.font: context.font]))
    bgColors.append(context.theme.lineBg(line))
    if let identity = DiffViewportLineIdentity.unified(line) {
        viewportLineLocations.append(DiffViewportLineLocation(
            identity: identity,
            characterRange: NSRange(
                location: lineStart,
                length: result.length - lineStart
            )
        ))
    }
}
