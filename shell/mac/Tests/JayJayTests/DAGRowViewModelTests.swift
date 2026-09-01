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

    func testCompareSourceRowMatchesByChangeIdOrCommitId() {
        let entry = makeEntry(
            changeId: "compare-source-change",
            commitId: "compare-source-commit",
            description: "feat-x",
            isImmutable: false
        )

        for compareFromId in ["compare-source-change", "compare-source-commit"] {
            let viewModel = DAGRowViewModel(
                entry: entry,
                layout: DAGLayout(entries: [entry]),
                index: 0,
                selectedId: "other-change",
                compareFromId: compareFromId,
                contextTargetId: nil,
                rebaseDrag: nil,
                rebasePreviewText: nil,
                bookmarkDrag: nil,
                bookmarkPreviewText: nil,
                colorScheme: .light
            )

            XCTAssertEqual(viewModel.selectionAccent, .compareSource)
            XCTAssertTrue(viewModel.isSelectionHighlighted)
        }
    }

    func testCombinedDiffParentIsNotHighlightedOutsideMultiSelection() {
        let entry = makeEntry(
            changeId: "combined-diff-parent",
            commitId: "combined-diff-parent-commit",
            description: "parent",
            isImmutable: false
        )

        let viewModel = DAGRowViewModel(
            entry: entry,
            layout: DAGLayout(entries: [entry]),
            index: 0,
            selectedId: "selected-head",
            selectedIds: ["selected-head", "selected-middle"],
            compareFromId: "combined-diff-parent",
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertNil(viewModel.selectionAccent)
        XCTAssertFalse(viewModel.isSelectionHighlighted)
    }

    func testWideGraphKeepsFixedGraphWidth() {
        let entries = [
            makeEntry(
                changeId: "merge-change",
                commitId: "merge",
                description: "merge",
                isImmutable: false,
                parents: ["p0", "p1", "p2", "p3", "p4", "p5"]
            ),
            makeEntry(
                changeId: "p5-change",
                commitId: "p5",
                description: "feature",
                isImmutable: false,
                parents: ["base"]
            ),
            makeEntry(
                changeId: "p4-change",
                commitId: "p4",
                description: "feature",
                isImmutable: false,
                parents: ["base"]
            )
        ]
        let layout = DAGLayout(entries: entries)

        let viewModel = DAGRowViewModel(
            entry: entries[1],
            layout: layout,
            index: 1,
            selectedId: nil,
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertEqual(layout.maxLanes(), 6)
        XCTAssertEqual(layout.displayLaneCount(), dagCompactVisibleLanes)
        XCTAssertEqual(layout.displayLane(for: layout.lane(for: "p5")), dagCompactVisibleLanes - 1)
        XCTAssertTrue(layout.hasLaneOverflow(at: 1))
        XCTAssertEqual(viewModel.graphWidth, layout.graphWidth)
    }

    func testFourLaneGraphKeepsDynamicGraphWidth() {
        let entries = [
            makeEntry(
                changeId: "merge-change",
                commitId: "merge",
                description: "merge",
                isImmutable: false,
                parents: ["p0", "p1", "p2", "p3"]
            ),
            makeEntry(
                changeId: "p3-change",
                commitId: "p3",
                description: "feature",
                isImmutable: false,
                parents: ["base"]
            )
        ]
        let layout = DAGLayout(entries: entries)

        let viewModel = DAGRowViewModel(
            entry: entries[1],
            layout: layout,
            index: 1,
            selectedId: nil,
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )

        XCTAssertEqual(layout.maxLanes(), 4)
        XCTAssertEqual(layout.displayLaneCount(), 4)
        XCTAssertEqual(layout.displayLane(for: layout.lane(for: "p3")), 3)
        XCTAssertFalse(layout.hasLaneOverflow(at: 1))
        XCTAssertEqual(viewModel.graphWidth, CGFloat(4) * laneWidth + 8)
    }

    private func makeEntry(
        changeId: String, commitId: String, description: String, isImmutable: Bool,
        isDivergent: Bool = false,
        parents: [String] = []
    ) -> GraphEntry {
        GraphEntry(
            change: mockChangeInfo(
                changeId: changeId,
                commitId: commitId,
                description: description,
                parents: parents,
                isImmutable: isImmutable,
                isDivergent: isDivergent
            ),
            edges: parents.map { GraphEdge(target: $0, edgeType: .direct) }
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
            isConflicted: false,
            startLocation: .zero,
            armedAt: nil,
            phase: .dragging,
            location: .zero,
            hoveredCommitId: hoveredCommitId
        )
    }
}
