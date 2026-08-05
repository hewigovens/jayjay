import AppKit
import JayJayCore
import SwiftUI

public extension NativeDiffView {
    func updateNSView(_ containerView: DiffTextContainerView, context: Context) {
        let gutterTextView = containerView.gutterTextView
        let textView = containerView.textView
        guard let gutterLayoutManager = gutterTextView.layoutManager as? DiffLayoutManager,
              let layoutManager = textView.layoutManager as? DiffLayoutManager
        else { return }

        let fontSize = fontSize
        let font = NSFont(name: fontFamily, size: fontSize) ?? .monospacedSystemFont(ofSize: fontSize, weight: .regular)
        let isDark = colorScheme == .dark
        let theme = DiffColors(isDark: isDark)
        textView.applyFindSelectionColors(theme)
        context.coordinator.onExpandContext = onExpandContext
        containerView.applySelectionResetGeneration(resetSelectionGeneration)
        let viewportAnchor = containerView.captureViewportAnchor()
        let selectionActions = gutterActions as? any DiffGutterSelectionActions
        let reviewActions = gutterActions as? any DiffGutterReviewActions
        let displayLines = displayLines ?? diffDisplayLines(lines: diff.lines)

        let gutterParagraphStyle = NSMutableParagraphStyle()
        let showsLineCheckboxes = selectionActions != nil
        let reviewModeEnabled = !showsLineCheckboxes && (reviewActions?.reviewModeEnabled == true)
        // Only diff-edit's per-line checkbox column needs its own slot; review state paints the group stripe in the always-present leftmost column.
        let showsCheckboxColumn = showsLineCheckboxes
        gutterParagraphStyle.alignment = .left
        let gutterAttrs: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: theme.gutterText,
            .paragraphStyle: gutterParagraphStyle
        ]

        var groupIndexAtLineNumber: [Int: UInt32] = [:]
        let groups = displayGroups ?? changeGroups(lines: displayLines)
        let groupsByIndex = Dictionary(uniqueKeysWithValues: groups.map { ($0.index, $0) })
        for group in groups {
            for lineNumber in Int(group.startLine) ... Int(group.endLine) {
                groupIndexAtLineNumber[lineNumber] = group.index
            }
        }

        let noteActions = gutterActions as? any DiffGutterNoteActions
        // The column is reserved whenever notes are possible, so adding or removing the first note never shifts the gutter.
        let loadsNoteMarkers = reviewModeEnabled && noteActions?.reviewNotesEnabled == true
        let showsNoteColumn = reserveNoteColumn || loadsNoteMarkers
        let noteSummariesByLine = loadsNoteMarkers
            ? noteSummariesByDisplayLine(displayLines: displayLines, groups: groups, notes: reviewNotes)
            : [:]
        let notedLines = Set(noteSummariesByLine.keys)
        let resolvedOnlyLines = Set(
            noteSummariesByLine.filter { $0.value.allSatisfy(\.isResolved) }.keys
        )
        let renderRows = diffRenderRows(displayLines: displayLines, notesByLine: noteSummariesByLine)
        let noteDotWidth = ("● " as NSString).size(withAttributes: [.font: font]).width
        let noteParagraphStyle = { (indent: String, isFirst: Bool, isLast: Bool) -> NSParagraphStyle in
            let style = NSMutableParagraphStyle()
            // Text starts at the anchor line's first character, inset past the bubble's rounded edge.
            let textStart = (indent as NSString).size(withAttributes: [.font: font]).width
                + DiffNoteBubbleMetrics.innerPadding
            style.firstLineHeadIndent = textStart
            style.headIndent = textStart
            if isFirst {
                style.paragraphSpacingBefore = DiffNoteBubbleMetrics.verticalSpacing
            }
            if isLast {
                style.paragraphSpacing = DiffNoteBubbleMetrics.verticalSpacing
            }
            return style
        }

        let result = NSMutableAttributedString()
        let groupWidth = (NativeDiffGutterRenderContext.groupColumnText as NSString)
            .size(withAttributes: [.font: font]).width
        let groupStripeWidth: CGFloat = 6
        let checkboxWidth = max(
            ("✓ " as NSString).size(withAttributes: [.font: font]).width,
            ("□ " as NSString).size(withAttributes: [.font: font]).width
        )
        let gutterHorizontalInset = gutterTextView.textContainerInset.width
        let gutterTrailingPadding: CGFloat = 10
        let measuredLineDigits = displayLines.reduce(into: 1) { digits, line in
            let lineNumber = max(line.oldLineNo ?? 0, line.newLineNo ?? 0)
            digits = max(digits, String(lineNumber).count)
        }
        let maxLineDigits = compactGutterWidth ? 1 : measuredLineDigits
        var contentLineBgColors: [NSColor] = []
        var viewportLineLocations: [DiffViewportLineLocation] = []

        var noteBubbleRanges: [NSRange] = []
        var noteBubbleStart = 0
        for row in renderRows {
            guard case let .line(line, _) = row else {
                if case let .note(text, indent, isFirst, isLast) = row {
                    if isFirst {
                        noteBubbleStart = result.length
                    }
                    result.append(NSAttributedString(string: "\(text)\n", attributes: [
                        .font: font,
                        .foregroundColor: NSColor.labelColor,
                        .paragraphStyle: noteParagraphStyle(indent, isFirst, isLast)
                    ]))
                    if isLast {
                        noteBubbleRanges.append(NSRange(
                            location: noteBubbleStart, length: result.length - noteBubbleStart
                        ))
                    }
                    // The bubble draws its own background; a full-width band here would read as part of the diff.
                    contentLineBgColors.append(.clear)
                }
                continue
            }
            appendNativeDiffLine(
                line,
                to: result,
                context: NativeDiffLineRenderContext(
                    font: font,
                    theme: theme,
                    enablesContextExpansion: onExpandContext != nil
                ),
                bgColors: &contentLineBgColors,
                viewportLineLocations: &viewportLineLocations
            )
        }

        if diff.lines.isEmpty {
            result.append(NSAttributedString(
                string: "No differences",
                attributes: [.font: font, .foregroundColor: NSColor.secondaryLabelColor]
            ))
            contentLineBgColors.append(.clear)
        }

        layoutManager.lineBgColors = contentLineBgColors
        layoutManager.lineStripeColors = renderRows.map { row in
            switch row {
                case let .line(line, _): conflictStripe(conflictKind: line.conflictKind, theme: theme)
                case .note: NSColor.clear
            }
        }
        layoutManager.noteBubbleRanges = noteBubbleRanges
        layoutManager.noteBubbleFill = theme.noteRowBg
        layoutManager.noteBubbleStroke = NSColor.systemOrange.withAlphaComponent(0.35)
        layoutManager.lineStripeX = 0
        layoutManager.lineStripeWidth = 3
        layoutManager.selectedRangeBgColor = .selectedTextBackgroundColor
        layoutManager.findMatchBgColor = .findHighlightColor
        gutterTextView.menuProvider = { selection in
            menuProvider(selection: selection, changeGroupsByIndex: groupsByIndex)
        }
        gutterTextView.groupRangeProvider = { lineNumber in
            expandedHunkRange(containing: lineNumber ... lineNumber)
        }
        if let selectionActions {
            gutterTextView.activateGroup = { selectionActions.selectChangeGroup($0) }
        } else {
            gutterTextView.activateGroup = nil
        }
        gutterTextView.groupHitWidth = groupWidth
        gutterTextView.toggleLineCheckbox = selectionActions.map { actions in
            { actions.toggleLineCheckbox($0) }
        }
        gutterTextView.checkboxHitStart = groupWidth
        gutterTextView.checkboxHitWidth = showsCheckboxColumn ? checkboxWidth : 0
        if reviewModeEnabled, let reviewActions {
            gutterTextView.groupIndexAtLineNumber = groupIndexAtLineNumber
            gutterTextView.toggleReviewCheckbox = { groupIdx in
                reviewActions.toggleHunkReviewed(groupIndex: groupIdx)
            }
        } else {
            gutterTextView.groupIndexAtLineNumber = [:]
            gutterTextView.toggleReviewCheckbox = nil
        }
        let noteColumnWidth = showsNoteColumn ? noteDotWidth : 0
        gutterTextView.notedLines = notedLines
        gutterTextView.noteHitStart = groupWidth
        gutterTextView.noteHitWidth = noteColumnWidth
        gutterTextView.onNoteClicked = noteActions.map { actions in
            { [weak gutterTextView] lineNumber, rect in
                guard let gutterTextView, let notes = noteSummariesByLine[lineNumber] else { return }
                presentReviewNotePopover(from: gutterTextView, at: rect, notes: notes, actions: actions)
            }
        }
        gutterLayoutManager.selectionHighlightLeadingInset = groupWidth + noteColumnWidth
            + (showsCheckboxColumn ? checkboxWidth : 0)
        gutterTextView.onSelectionChanged = { selection in
            gutterActions?.didSelectLines(selection.lineRange)
        }
        textView.textStorage?.setAttributedString(result)
        containerView.setViewportLineLocations(
            viewportLineLocations,
            restoring: viewportAnchor
        )
        containerView.scheduleRevealFeedback(
            revealFeedback,
            reduceMotion: reduceMotion
        )

        let renderGutter = { [weak containerView] in
            guard let containerView else { return }

            let logicalLineCount = max(renderRows.count, 1)
            let gutterWidth = renderWrappedGutter(
                gutterTextView: gutterTextView,
                gutterLayoutManager: gutterLayoutManager,
                context: NativeDiffGutterRenderContext(
                    content: .init(
                        lines: displayLines,
                        rows: renderRows,
                        visualLineCounts: layoutManager.visualLineCounts(logicalLineCount: logicalLineCount)
                    ),
                    style: .init(
                        font: font,
                        theme: theme,
                        gutterAttrs: gutterAttrs,
                        gutterParagraphStyle: gutterParagraphStyle,
                        maxLineDigits: maxLineDigits
                    ),
                    layout: .init(
                        groupStripeWidth: groupStripeWidth,
                        gutterHorizontalInset: gutterHorizontalInset,
                        gutterTrailingPadding: gutterTrailingPadding,
                        showsCheckboxColumn: showsCheckboxColumn,
                        showsNoteColumn: showsNoteColumn
                    ),
                    review: .init(
                        reviewModeEnabled: reviewModeEnabled,
                        groupIndexAtLineNumber: groupIndexAtLineNumber,
                        reviewActions: reviewActions,
                        notedLines: notedLines,
                        resolvedOnlyLines: resolvedOnlyLines,
                        currentSelectedLineRange: gutterActions?.currentSelectedLineRange
                    )
                )
            )
            let targetWidth = compactGutterWidth
                ? DiffGutterMetrics.richPreviewWidth(
                    font: font,
                    showsNoteColumn: showsNoteColumn,
                    hasVisibleNoteMarker: !notedLines.isEmpty
                )
                : max(DiffGutterMetrics.minimumUnifiedWidth, gutterWidth)
            containerView.updateGutterWidth(targetWidth)
        }

        containerView.onContentLayoutChanged = renderGutter
        containerView.onContentHeightChanged = onContentHeightChanged
        containerView.setFitsContent(onContentHeightChanged != nil)
        renderGutter()
        containerView.reportContentHeightIfNeeded()
    }
}
