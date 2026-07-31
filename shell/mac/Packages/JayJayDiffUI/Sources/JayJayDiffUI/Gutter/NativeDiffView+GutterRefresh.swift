import JayJayCore

extension NativeDiffView {
    func refreshSelectionGutter(
        containerView: DiffTextContainerView,
        gutterLayoutManager: DiffLayoutManager,
        layoutManager: DiffLayoutManager,
        selectionActions: any DiffGutterSelectionActions,
        cache: NativeDiffContextCoordinator.SelectionRenderCache
    ) {
        configureGutterInteractions(
            containerView.gutterTextView,
            groupsByIndex: cache.groupsByIndex,
            selectionActions: selectionActions
        )
        installGutterRenderer(
            containerView: containerView,
            gutterLayoutManager: gutterLayoutManager,
            layoutManager: layoutManager,
            context: cache.gutterContext
        )
    }

    func configureGutterInteractions(
        _ gutterTextView: DiffGutterTextView,
        groupsByIndex: [UInt32: ChangeGroup],
        selectionActions: (any DiffGutterSelectionActions)?
    ) {
        gutterTextView.menuProvider = { selection in
            menuProvider(selection: selection, changeGroupsByIndex: groupsByIndex)
        }
        gutterTextView.groupRangeProvider = { lineNumber in
            expandedHunkRange(containing: lineNumber ... lineNumber)
        }
        gutterTextView.activateGroup = selectionActions.map { actions in
            { actions.selectChangeGroup($0) }
        }
        gutterTextView.toggleLineCheckbox = selectionActions.map { actions in
            { actions.toggleLineCheckbox($0) }
        }
        gutterTextView.onSelectionChanged = { selection in
            gutterActions?.didSelectLines(selection.lineRange)
        }
    }

    func installGutterRenderer(
        containerView: DiffTextContainerView,
        gutterLayoutManager: DiffLayoutManager,
        layoutManager: DiffLayoutManager,
        context baseContext: NativeDiffGutterRenderContext
    ) {
        let renderGutter = { [weak containerView] in
            guard let containerView else { return }
            let logicalLineCount = max(baseContext.content.rows.count, 1)
            let context = NativeDiffGutterRenderContext(
                content: .init(
                    lines: baseContext.content.lines,
                    rows: baseContext.content.rows,
                    visualLineCounts: layoutManager.visualLineCounts(logicalLineCount: logicalLineCount)
                ),
                style: baseContext.style,
                layout: baseContext.layout,
                review: baseContext.review
            )
            let gutterWidth = renderWrappedGutter(
                gutterTextView: containerView.gutterTextView,
                gutterLayoutManager: gutterLayoutManager,
                context: context
            )
            let targetWidth = compactGutterWidth
                ? DiffGutterMetrics.richPreviewWidth(
                    font: context.style.font,
                    showsNoteColumn: context.layout.showsNoteColumn,
                    hasVisibleNoteMarker: !context.review.notedLines.isEmpty
                )
                : max(DiffGutterMetrics.minimumUnifiedWidth, gutterWidth)
            containerView.updateGutterWidth(targetWidth)
        }
        containerView.onContentLayoutChanged = renderGutter
        renderGutter()
    }
}
