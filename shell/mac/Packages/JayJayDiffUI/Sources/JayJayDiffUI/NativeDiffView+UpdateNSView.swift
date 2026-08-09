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
        context.coordinator.onExpandContext = onExpandContext
        containerView.applySelectionResetGeneration(resetSelectionGeneration)
        let selectionActions = gutterActions as? any DiffGutterSelectionActions
        let reviewActions = gutterActions as? any DiffGutterReviewActions
        let selectionRenderIdentity: NativeDiffContextCoordinator.SelectionRenderCache.Identity? = selectionActions.flatMap { actions in
            guard let contentGeneration else { return nil }
            return NativeDiffContextCoordinator.SelectionRenderCache.Identity(
                contentGeneration: contentGeneration,
                reserveNoteColumn: reserveNoteColumn,
                compactGutterWidth: compactGutterWidth,
                enablesContextExpansion: onExpandContext != nil,
                resetSelectionGeneration: resetSelectionGeneration,
                revealFeedback: revealFeedback,
                isDark: isDark,
                fontSize: fontSize,
                fontFamily: fontFamily,
                reduceMotion: reduceMotion,
                fitsContent: onContentHeightChanged != nil,
                currentSelectedLineRange: actions.currentSelectedLineRange
            )
        }
        if let selectionRenderIdentity,
           let selectionRenderCache = context.coordinator.selectionRenderCache,
           selectionRenderCache.identity == selectionRenderIdentity,
           let selectionActions
        {
            refreshSelectionGutter(
                containerView: containerView,
                gutterLayoutManager: gutterLayoutManager,
                layoutManager: layoutManager,
                selectionActions: selectionActions,
                cache: selectionRenderCache
            )
            containerView.onContentHeightChanged = onContentHeightChanged
            containerView.setFitsContent(onContentHeightChanged != nil)
            return
        }
        context.coordinator.selectionRenderCache = nil

        let theme = DiffColors(isDark: isDark)
        textView.applyFindSelectionColors(theme)
        let viewportAnchor = containerView.captureViewportAnchor()
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
        configureGutterInteractions(
            gutterTextView,
            groupsByIndex: groupsByIndex,
            selectionActions: selectionActions
        )
        gutterTextView.groupHitWidth = groupWidth
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
        textView.textStorage?.setAttributedString(result)
        containerView.setViewportLineLocations(
            viewportLineLocations,
            restoring: viewportAnchor
        )
        containerView.scheduleRevealFeedback(
            revealFeedback,
            reduceMotion: reduceMotion
        )

        let gutterContext = NativeDiffGutterRenderContext(
            content: .init(
                lines: displayLines,
                rows: renderRows,
                visualLineCounts: []
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
                showsNoteColumn: showsNoteColumn,
                showsChangeMarkers: showsChangeMarkers
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
        if let selectionRenderIdentity {
            context.coordinator.selectionRenderCache = .init(
                identity: selectionRenderIdentity,
                gutterContext: gutterContext,
                groupsByIndex: groupsByIndex
            )
        }

        containerView.onContentHeightChanged = onContentHeightChanged
        containerView.setFitsContent(onContentHeightChanged != nil)
        installGutterRenderer(
            containerView: containerView,
            gutterLayoutManager: gutterLayoutManager,
            layoutManager: layoutManager,
            context: gutterContext
        )
        containerView.reportContentHeightIfNeeded()
    }
}
