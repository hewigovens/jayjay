import AppKit
import JayJayCore

extension NativeDiffView {
    func menuProvider(selection: DiffGutterSelection) -> [DiffGutterMenuItem] {
        guard let gutterActions else { return [] }

        var items: [DiffGutterMenuItem] = []
        if let selectionActions = gutterActions as? any DiffGutterSelectionActions,
           let hunkRange = expandedHunkRange(containing: selection.lineRange)
        {
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

    func expandedHunkRange(containing selection: ClosedRange<Int>) -> ClosedRange<Int>? {
        DiffGutterGrouping.expandedChangedRange(in: diff.lines, containing: selection)
    }

    func spanBackground(span: DiffSpan, theme: DiffColors) -> NSColor {
        switch span.style {
            case .added: theme.addedWordBg
            case .removed: theme.removedWordBg
            default: .clear
        }
    }

    func groupText() -> String {
        "  "
    }

    func groupStripeColor(for line: DiffLine, groupRange: ClosedRange<Int>?, theme: DiffColors) -> NSColor {
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
