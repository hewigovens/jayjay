import JayJayCore

struct MergeSourceHighlights {
    let spans: [[DiffSpan]]
    let lineStyles: [DiffSpanStyle]

    init(diffLines: [DiffLine]) {
        spans = diffLines.map(\.spans)
        lineStyles = diffLines.map(\.style)
    }

    init(syntaxSpans: [[DiffSpan]]) {
        spans = syntaxSpans
        lineStyles = Array(repeating: .context, count: syntaxSpans.count)
    }
}

struct MergeEditorHighlights {
    let left: MergeSourceHighlights
    let base: MergeSourceHighlights
    let right: MergeSourceHighlights
    let resultText: String
    let result: [[DiffSpan]]
    let hunks: [MergeHunkHighlights]

    init(
        path: String,
        left: String,
        base: String,
        right: String,
        result: String,
        hunks: [MergeEditorHunk]
    ) {
        self.left = MergeSourceHighlights(diffLines: highlightFileAgainstBase(
            path: path,
            base: base,
            content: left
        ))
        self.base = MergeSourceHighlights(syntaxSpans: highlightFileLines(path: path, content: base))
        self.right = MergeSourceHighlights(diffLines: highlightFileAgainstBase(
            path: path,
            base: base,
            content: right
        ))
        resultText = result
        self.result = highlightFileLines(path: path, content: result)
        self.hunks = hunks.map { MergeHunkHighlights(path: path, result: result, hunk: $0) }
    }
}

struct MergeHunkHighlights: Identifiable {
    let hunk: MergeEditorHunk
    let unified: FileDiff

    var id: UInt32 {
        hunk.index
    }

    init(path: String, result: String, hunk: MergeEditorHunk) {
        self.hunk = hunk
        unified = mergeHunkDisplayDiff(path: path, result: result, hunk: hunk)
    }
}
