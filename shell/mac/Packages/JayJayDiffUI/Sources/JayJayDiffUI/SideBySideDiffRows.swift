import JayJayCore

struct SideBySideRow {
    var oldLineNo: String
    var oldMarker: String
    var oldSpans: [DiffSpan]
    var oldStyle: DiffSpanStyle
    var newLineNo: String
    var newMarker: String
    var newSpans: [DiffSpan]
    var newStyle: DiffSpanStyle
}

func buildRows(from lines: [DiffLine]) -> [SideBySideRow] {
    var rows: [SideBySideRow] = []
    var index = 0
    while index < lines.count {
        let line = lines[index]
        switch line.style {
            case .context:
                rows.append(SideBySideRow(
                    oldLineNo: line.oldLineNo.map(String.init) ?? "",
                    oldMarker: " ",
                    oldSpans: line.spans,
                    oldStyle: .context,
                    newLineNo: line.newLineNo.map(String.init) ?? "",
                    newMarker: " ",
                    newSpans: line.spans,
                    newStyle: .context
                ))
                index += 1
            case .separator:
                rows.append(SideBySideRow(
                    oldLineNo: "",
                    oldMarker: "",
                    oldSpans: line.spans,
                    oldStyle: .separator,
                    newLineNo: "",
                    newMarker: "",
                    newSpans: line.spans,
                    newStyle: .separator
                ))
                index += 1
            case .removed:
                var removed: [DiffLine] = []
                while index < lines.count, lines[index].style == .removed {
                    removed.append(lines[index])
                    index += 1
                }
                var added: [DiffLine] = []
                while index < lines.count, lines[index].style == .added {
                    added.append(lines[index])
                    index += 1
                }
                for pairIndex in 0 ..< max(removed.count, added.count) {
                    let removedLine = pairIndex < removed.count ? removed[pairIndex] : nil
                    let addedLine = pairIndex < added.count ? added[pairIndex] : nil
                    rows.append(SideBySideRow(
                        oldLineNo: removedLine?.oldLineNo.map(String.init) ?? "",
                        oldMarker: removedLine != nil ? "-" : " ",
                        oldSpans: removedLine?.spans ?? [],
                        oldStyle: removedLine != nil ? .removed : .context,
                        newLineNo: addedLine?.newLineNo.map(String.init) ?? "",
                        newMarker: addedLine != nil ? "+" : " ",
                        newSpans: addedLine?.spans ?? [],
                        newStyle: addedLine != nil ? .added : .context
                    ))
                }
            case .added:
                rows.append(SideBySideRow(
                    oldLineNo: "",
                    oldMarker: " ",
                    oldSpans: [],
                    oldStyle: .context,
                    newLineNo: line.newLineNo.map(String.init) ?? "",
                    newMarker: "+",
                    newSpans: line.spans,
                    newStyle: .added
                ))
                index += 1
            default:
                index += 1
        }
    }
    return rows
}
