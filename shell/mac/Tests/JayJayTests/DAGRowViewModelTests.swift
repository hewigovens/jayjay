@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

final class DAGRowViewModelTests: XCTestCase {
    func testPressingSourceHidesDragAffordances() {
        let entry = makeEntry(
            changeId: "source-change",
            commitId: "source-commit",
            description: "feat-x",
            isImmutable: false
        )

        let viewModel = DAGRowViewModel(
            entry: entry,
            layout: DAGLayout(entries: [entry]),
            index: 0,
            selectedId: nil,
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: makeDragState(sourceCommitId: "source-commit", phase: .pressing),
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertFalse(viewModel.isRebaseSource)
        XCTAssertFalse(viewModel.isRebaseArmed)
        XCTAssertNil(viewModel.dragTargetText)
        XCTAssertEqual(viewModel.wiggleAngle(at: Date()), 0)
    }

    func testArmedSourceShowsDragAffordances() {
        let entry = makeEntry(
            changeId: "source-change",
            commitId: "source-commit",
            description: "feat-x",
            isImmutable: false
        )
        let armedAt = Date(timeIntervalSinceReferenceDate: 10)

        let viewModel = DAGRowViewModel(
            entry: entry,
            layout: DAGLayout(entries: [entry]),
            index: 0,
            selectedId: nil,
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: makeDragState(
                sourceCommitId: "source-commit",
                phase: .armed,
                armedAt: armedAt
            ),
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertTrue(viewModel.isRebaseSource)
        XCTAssertTrue(viewModel.isRebaseArmed)
        XCTAssertEqual(viewModel.dragTargetText, "Drag to choose a new parent")
        XCTAssertNotEqual(viewModel.wiggleAngle(at: armedAt.addingTimeInterval(0.2)), 0)
    }

    func testHoverTargetShowsPreview() {
        let source = makeEntry(
            changeId: "source-change",
            commitId: "source-commit",
            description: "feat-x",
            isImmutable: false
        )
        let target = makeEntry(
            changeId: "target-change",
            commitId: "target-commit",
            description: "main update",
            isImmutable: true
        )

        let viewModel = DAGRowViewModel(
            entry: target,
            layout: DAGLayout(entries: [source, target]),
            index: 1,
            selectedId: nil,
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: makeDragState(sourceCommitId: "source-commit", phase: .dragging),
            rebasePreviewText: "Rebase feat-x onto main?",
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .dark
        )

        XCTAssertTrue(viewModel.isRebaseCandidate)
        XCTAssertTrue(viewModel.isRebaseHoverTarget)
        XCTAssertEqual(viewModel.dragTargetText, "Rebase feat-x onto main?")
        XCTAssertTrue(viewModel.showsReturnHint)
    }

    func testBookmarkDragHoverShowsDropTarget() {
        let target = makeEntry(
            changeId: "target-change",
            commitId: "target-commit",
            description: "main update",
            isImmutable: false
        )

        let viewModel = DAGRowViewModel(
            entry: target,
            layout: DAGLayout(entries: [target]),
            index: 0,
            selectedId: nil,
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: makeBookmarkDrag(hoveredCommitId: "target-commit"),
            bookmarkPreviewText: "Move feature here?",
            colorScheme: .light
        )

        XCTAssertTrue(viewModel.isHoverDropTarget)
        XCTAssertEqual(viewModel.outlineState, .hoverTarget)
        XCTAssertEqual(viewModel.dragTargetText, "Move feature here?")
        XCTAssertTrue(viewModel.showsReturnHint)
    }

    func testBookmarkDragBeforePreviewDelayStillHighlights() {
        // Hovered, but the preview delay hasn't elapsed: highlight + generic bubble,
        // no Return hint yet.
        let target = makeEntry(
            changeId: "target-change",
            commitId: "target-commit",
            description: "main update",
            isImmutable: false
        )

        let viewModel = DAGRowViewModel(
            entry: target,
            layout: DAGLayout(entries: [target]),
            index: 0,
            selectedId: nil,
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: makeBookmarkDrag(hoveredCommitId: "target-commit"),
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertTrue(viewModel.isHoverDropTarget)
        XCTAssertEqual(viewModel.dragTargetText, "Release to move here")
        XCTAssertFalse(viewModel.showsReturnHint)
    }

    func testBookmarkDragNonHoveredRowHasNoDropTarget() {
        let target = makeEntry(
            changeId: "target-change",
            commitId: "target-commit",
            description: "main update",
            isImmutable: false
        )

        let viewModel = DAGRowViewModel(
            entry: target,
            layout: DAGLayout(entries: [target]),
            index: 0,
            selectedId: nil,
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: makeBookmarkDrag(hoveredCommitId: "other-commit"),
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertFalse(viewModel.isHoverDropTarget)
        XCTAssertNil(viewModel.dragTargetText)
        XCTAssertFalse(viewModel.showsReturnHint)
    }

    func testSelectedRowKeepsSelectionAccent() {
        let entry = makeEntry(
            changeId: "selected-change",
            commitId: "selected-commit",
            description: "feat-x",
            isImmutable: false
        )

        let viewModel = DAGRowViewModel(
            entry: entry,
            layout: DAGLayout(entries: [entry]),
            index: 0,
            selectedId: "selected-change",
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertEqual(viewModel.selectionAccent, .selected)
        XCTAssertEqual(viewModel.leadingAccentColor, .accentColor)
        XCTAssertNil(viewModel.dragTargetText)
    }

    func testDivergentSelectedRowMatchesCommitId() {
        let entry = makeEntry(
            changeId: "same-change",
            commitId: "selected-commit",
            description: "feat-x",
            isImmutable: false,
            isDivergent: true
        )

        let viewModel = DAGRowViewModel(
            entry: entry,
            layout: DAGLayout(entries: [entry]),
            index: 0,
            selectedId: "selected-commit",
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertEqual(viewModel.selectionAccent, .selected)
    }

    func testDivergentRowDoesNotMatchSharedChangeIdSelection() {
        let entry = makeEntry(
            changeId: "same-change",
            commitId: "selected-commit",
            description: "feat-x",
            isImmutable: false,
            isDivergent: true
        )

        let viewModel = DAGRowViewModel(
            entry: entry,
            layout: DAGLayout(entries: [entry]),
            index: 0,
            selectedId: "same-change",
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertNil(viewModel.selectionAccent)
    }

    private func makeEntry(
        changeId: String,
        commitId: String,
        description: String,
        isImmutable: Bool,
        isDivergent: Bool = false
    ) -> GraphEntry {
        GraphEntry(
            change: ChangeInfo(
                changeId: ShortId(id: changeId, shortLen: 1),
                commitId: ShortId(id: commitId, shortLen: 1),
                description: description,
                author: .tester,
                parents: [],
                bookmarks: [],
                remoteBookmarks: [],
                tags: [],
                isWorkingCopy: false,
                hasConflict: false,
                isEmpty: false,
                isImmutable: isImmutable,
                isDivergent: isDivergent
            ),
            edges: []
        )
    }

    private func makeDragState(
        sourceCommitId: String,
        phase: DAGRebasePhase,
        armedAt: Date? = nil
    ) -> DAGRebaseDragState {
        DAGRebaseDragState(
            sourceCommitId: sourceCommitId,
            sourceChangeId: "source-change",
            sourceRev: "source-change",
            sourceLabel: "feat-x",
            startLocation: .zero,
            armedAt: armedAt,
            phase: phase,
            location: .zero,
            hoveredCommitId: "target-commit"
        )
    }

    private func makeBookmarkDrag(hoveredCommitId: String?) -> BookmarkDragState {
        BookmarkDragState(
            bookmarkName: "feature",
            sourceCommitId: "source-commit",
            startLocation: .zero,
            armedAt: nil,
            phase: .dragging,
            location: .zero,
            hoveredCommitId: hoveredCommitId
        )
    }
}
