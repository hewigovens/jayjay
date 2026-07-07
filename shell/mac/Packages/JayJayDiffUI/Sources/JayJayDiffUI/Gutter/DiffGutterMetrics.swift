import AppKit
import JayJayCore

public enum DiffGutterMetrics {
    public static let minimumUnifiedWidth: CGFloat = 52
    static let groupColumnText = "   "

    public static func unifiedWidth(
        displayLines: [DiffLine],
        font: NSFont,
        showsCheckboxColumn: Bool = false,
        showsNoteColumn: Bool = false,
        hasVisibleNoteMarker: Bool = false
    ) -> CGFloat {
        let maxLineDigits = displayLines.reduce(into: 1) { digits, line in
            let lineNumber = max(line.oldLineNo ?? 0, line.newLineNo ?? 0)
            digits = max(digits, String(lineNumber).count)
        }
        let noteColumn = showsNoteColumn ? (hasVisibleNoteMarker ? "● " : "  ") : ""
        let checkboxColumn = showsCheckboxColumn ? widestCheckboxColumn(font: font) : ""
        let lineNumberText = String(repeating: "0", count: maxLineDigits)
        let gutterText = "\(groupColumnText)\(noteColumn)\(checkboxColumn)\(lineNumberText) \(lineNumberText)\n"
        let gutterTextWidth = (gutterText as NSString).size(withAttributes: [.font: font]).width
        return max(minimumUnifiedWidth, ceil(8 + gutterTextWidth + 10 + 8))
    }

    private static func widestCheckboxColumn(font: NSFont) -> String {
        let checked = ("✓ " as NSString).size(withAttributes: [.font: font]).width
        let unchecked = ("□ " as NSString).size(withAttributes: [.font: font]).width
        return checked >= unchecked ? "✓ " : "□ "
    }
}
