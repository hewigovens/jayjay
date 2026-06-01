import AppKit
import JayJayCore

extension DiffLine {
    var rawText: String {
        spans.rawText
    }
}

extension [DiffSpan] {
    var rawText: String {
        map(\.text).joined()
    }
}

func conflictLabel(for line: DiffLine) -> String? {
    switch line.conflictKind {
        case .start, .end, .section:
            conflictDisplayText(kind: line.conflictKind, raw: line.rawText)
        default:
            nil
    }
}

func conflictLabel(spans: [DiffSpan], kind: ConflictLineKind) -> String? {
    switch kind {
        case .start, .end, .section:
            conflictDisplayText(kind: kind, raw: spans.rawText)
        default:
            nil
    }
}

func conflictDisplayLine(label: String, kind: ConflictLineKind) -> String {
    switch kind {
        case .section:
            "    \(label)"
        default:
            "  \(label)"
    }
}

func conflictLineAttributes(
    kind: ConflictLineKind,
    font: NSFont,
    theme: DiffColors
) -> [NSAttributedString.Key: Any] {
    let color = switch kind {
        case .start:
            theme.conflictHeaderText
        case .end, .section:
            theme.conflictSectionText
        case .added:
            theme.addedText
        case .removed:
            theme.removedText
        default:
            theme.contextText
    }
    return [
        NSAttributedString.Key.font: font,
        NSAttributedString.Key.foregroundColor: color
    ]
}
