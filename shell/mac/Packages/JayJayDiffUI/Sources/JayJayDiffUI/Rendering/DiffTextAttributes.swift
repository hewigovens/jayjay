import AppKit
import JayJayCore

extension NSAttributedString.Key {
    static let diffWordHighlightColor = NSAttributedString.Key("jayjay.diff.wordHighlightColor")
}

func diffSpanAttributes(
    for span: DiffSpan,
    lineStyle: DiffSpanStyle,
    conflictKind: ConflictLineKind,
    font: NSFont,
    theme: DiffColors
) -> [NSAttributedString.Key: Any] {
    var attrs: [NSAttributedString.Key: Any] = [
        .font: font,
        .foregroundColor: theme.spanText(span, lineStyle: lineStyle, conflictKind: conflictKind)
    ]
    let wordBg = theme.spanBackground(span)
    if wordBg != .clear { attrs[.diffWordHighlightColor] = wordBg }
    return attrs
}
