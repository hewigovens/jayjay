import Foundation
import JayJayCore

/// How a pre-wrapped visual row contributes to copied text, so Copy yields the source lines rather than the screen rows.
enum DiffCopyRow {
    case line
    case wrapped
    case padding

    static func rows(_ wrapped: [WrappedSbsRow], side: KeyPath<SideBySideRow, RowSide>) -> [DiffCopyRow] {
        wrapped.indices.map { ix in
            let row = wrapped[ix]
            guard !row.row.fullWidth else { return .line }
            let rowSide = row.row[keyPath: side]
            if rowSide.lineNo.isEmpty, rowSide.spans.isEmpty {
                return .padding
            }
            if ix + 1 < wrapped.count, wrapped[ix + 1].rowIx == row.rowIx, !wrapped[ix + 1].row[keyPath: side].spans.isEmpty {
                return .wrapped
            }
            return .line
        }
    }
}

extension NSAttributedString {
    func copyText(in range: NSRange, rows: [DiffCopyRow]) -> NSAttributedString {
        let text = NSMutableAttributedString()
        let starts = (string as NSString).lineStarts()
        for (ix, start) in starts.enumerated() {
            if start >= NSMaxRange(range) {
                break
            }
            let end = ix + 1 < starts.count ? starts[ix + 1] : length
            var selected = NSIntersectionRange(NSRange(location: start, length: end - start), range)
            let kind = ix < rows.count ? rows[ix] : .line
            guard selected.length > 0, kind != .padding else { continue }
            if kind == .wrapped, NSMaxRange(selected) == end {
                selected.length -= 1
            }
            text.append(attributedSubstring(from: selected))
        }
        return text
    }
}
