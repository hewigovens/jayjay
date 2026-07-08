import AppKit
import JayJayCore

extension NativeDiffView {
    func menuProvider(
        selection: DiffGutterSelection,
        changeGroupsByIndex: [UInt32: ChangeGroup]
    ) -> [DiffGutterMenuItem] {
        guard let gutterActions else { return [] }

        var items: [DiffGutterMenuItem] = []
        let displayLines = diffDisplayLines(lines: diff.lines)
        if let noteActions = gutterActions as? any DiffGutterNoteActions,
           noteActions.reviewNotesEnabled,
           let anchor = noteAnchor(
               at: selection.menuLineNumber,
               lines: displayLines,
               groups: changeGroupsByIndex
           )
        {
            let noteIds = noteActions.activeNotes(anchor: anchor).map(\.id)
            if noteIds.isEmpty {
                items.append(DiffGutterMenuItem(
                    title: "Add Review Note",
                    enabled: true,
                    action: { noteActions.addNote(anchor: anchor) }
                ))
            }
            if let noteId = noteIds.last {
                if !items.isEmpty {
                    items.append(.separator)
                }
                items.append(DiffGutterMenuItem(
                    title: "Edit Review Note",
                    enabled: true,
                    action: { noteActions.editNote(id: noteId) }
                ))
                items.append(DiffGutterMenuItem(
                    title: "Resolve Review Note",
                    enabled: true,
                    action: { noteActions.resolveNote(id: noteId) }
                ))
                items.append(DiffGutterMenuItem(
                    title: "Delete Review Note",
                    enabled: true,
                    action: { noteActions.deleteNote(id: noteId) }
                ))
            }
        }
        if let selectionActions = gutterActions as? any DiffGutterSelectionActions,
           let hunkRange = expandedHunkRange(containing: selection.lineRange)
        {
            if !items.isEmpty {
                items.append(.separator)
            }
            items.append(
                DiffGutterMenuItem(
                    title: "Select Change Group",
                    enabled: true,
                    action: { selectionActions.selectChangeGroup(hunkRange) }
                )
            )
        }
        if let selectionActions = gutterActions as? any DiffGutterSelectionActions {
            items.append(
                DiffGutterMenuItem(title: "Select File", enabled: true, action: { selectionActions.selectFile() })
            )
        }
        if let editActions = gutterActions as? any DiffGutterEditActions,
           editActions.canOpenDiffEdit
        {
            items.append(
                DiffGutterMenuItem(title: "Open Diff Edit Mode", enabled: true, action: { editActions.openDiffEdit() })
            )
        }
        if let editActions = gutterActions as? any DiffGutterEditActions,
           editActions.canAbandonSelectedLines
        {
            if !items.isEmpty {
                items.append(.separator)
            }
            let groupRange = expandedHunkRange(containing: selection.lineRange)
            let isWholeGroup = groupRange == selection.lineRange && selection.changedLineCount > 1
            items.append(
                DiffGutterMenuItem(
                    title: isWholeGroup ? "Abandon Change Group" : "Abandon Selected Lines",
                    enabled: selection.changedLineCount > 0,
                    action: selection.changedLineCount > 0 ? {
                        editActions.abandonSelectedLines(in: selection.lineRange)
                    } : nil
                )
            )
        }
        if !items.isEmpty,
           items.last?.action == nil,
           items.last?.title.isEmpty == true
        {
            _ = items.popLast()
        }

        return items
    }

    func noteAnchorsByLineNumber(
        lines: [DiffLine],
        groups: [UInt32: ChangeGroup]
    ) -> [Int: DiffReviewNoteAnchor] {
        groups.values.reduce(into: [Int: DiffReviewNoteAnchor]()) { result, group in
            for lineNumber in Int(group.startLine) ... Int(group.endLine) {
                if let anchor = noteAnchor(at: lineNumber, lines: lines, group: group) {
                    result[lineNumber] = anchor
                }
            }
        }
    }

    /// Matches notes on side + file line only — no anchor or excerpt construction — since gutter markers just need a position.
    func noteSummariesByDisplayLine(
        displayLines: [DiffLine],
        groups: [ChangeGroup],
        notes: [DiffReviewNoteSummary]
    ) -> [Int: [DiffReviewNoteSummary]] {
        guard !notes.isEmpty else { return [:] }
        struct AnchorKey: Hashable {
            let old: Bool
            let line: UInt32
        }
        var notesByAnchor: [AnchorKey: [DiffReviewNoteSummary]] = [:]
        for note in notes {
            guard let side = note.side, let line = note.line else { continue }
            notesByAnchor[AnchorKey(old: side == .old, line: line), default: []].append(note)
        }

        var result: [Int: [DiffReviewNoteSummary]] = [:]
        for group in groups {
            for lineNumber in Int(group.startLine) ... Int(group.endLine) {
                guard displayLines.indices.contains(lineNumber - 1) else { continue }
                let line = displayLines[lineNumber - 1]
                let key: AnchorKey? = switch line.style {
                    case .added: line.newLineNo.map { AnchorKey(old: false, line: $0) }
                    case .removed: line.oldLineNo.map { AnchorKey(old: true, line: $0) }
                    default: nil
                }
                if let key, let matches = notesByAnchor[key] {
                    result[lineNumber] = matches
                }
            }
        }
        return result
    }

    private func noteAnchor(
        at lineNumber: Int,
        lines: [DiffLine],
        groups: [UInt32: ChangeGroup]
    ) -> DiffReviewNoteAnchor? {
        let group = groups.values.first { group in
            let range = Int(group.startLine) ... Int(group.endLine)
            return range.contains(lineNumber)
        }
        guard let group else { return nil }
        return noteAnchor(at: lineNumber, lines: lines, group: group)
    }

    private func noteAnchor(
        at lineNumber: Int,
        lines: [DiffLine],
        group: ChangeGroup
    ) -> DiffReviewNoteAnchor? {
        guard lineNumber > 0,
              lines.indices.contains(lineNumber - 1)
        else { return nil }
        let line = lines[lineNumber - 1]
        switch line.style {
            case .added:
                guard let newLineNo = line.newLineNo else { return nil }
                return DiffReviewNoteAnchor(
                    groupIndex: group.index,
                    displayLine: UInt32(lineNumber),
                    side: .new,
                    line: newLineNo,
                    excerpt: line.rawText,
                    context: group.anchorContext
                )
            case .removed:
                guard let oldLineNo = line.oldLineNo else { return nil }
                return DiffReviewNoteAnchor(
                    groupIndex: group.index,
                    displayLine: UInt32(lineNumber),
                    side: .old,
                    line: oldLineNo,
                    excerpt: line.rawText,
                    context: group.anchorContext
                )
            default:
                return nil
        }
    }

    func expandedHunkRange(containing selection: ClosedRange<Int>) -> ClosedRange<Int>? {
        DiffGutterGrouping.expandedChangedRange(in: diffDisplayLines(lines: diff.lines), containing: selection)
    }

    func groupStripeColor(for line: DiffLine, groupRange: ClosedRange<Int>?, theme: DiffColors) -> NSColor {
        if line.conflictKind != .none {
            return theme.conflictStripe
        }
        guard line.isChanged,
              let groupRange,
              groupRange.upperBound > groupRange.lowerBound
        else { return .clear }
        return theme.groupStripe
    }

    func checkboxText(for lineNumber: Int, line: DiffLine) -> String {
        guard line.isChanged else { return "  " }
        guard let state = (gutterActions as? any DiffGutterSelectionActions)?.lineCheckboxState(for: lineNumber) else {
            return "  "
        }
        switch state {
            case .selected:
                return "✓ "
            case .unselected:
                return "□ "
        }
    }

    func checkboxColor(for lineNumber: Int, theme: DiffColors) -> NSColor {
        guard let state = (gutterActions as? any DiffGutterSelectionActions)?.lineCheckboxState(for: lineNumber) else {
            return theme.gutterText
        }
        switch state {
            case .selected:
                return .controlAccentColor
            case .unselected:
                return theme.gutterText
        }
    }

    func pad(_ value: String, toWidth width: Int) -> String {
        guard value.count < width else { return value }
        return String(repeating: " ", count: width - value.count) + value
    }
}

private extension DiffLine {
    var isChanged: Bool {
        style == .added || style == .removed
    }
}
