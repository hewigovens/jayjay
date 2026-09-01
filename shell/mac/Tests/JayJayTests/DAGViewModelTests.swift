@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

final class DAGViewModelTests: XCTestCase {
    func testTracksHoveredContextTarget() {
        let entry = makeEntry(changeId: "hovered", commitId: "hovered-commit", isDivergent: false)
        let viewModel = makeViewModel(entries: [entry], selectedId: "selected", contextTargetId: nil)

        XCTAssertEqual(viewModel.nextContextTargetId(hovering: true, entry: entry), "hovered")
    }

    func testReturnsSelectedRevisionsInVisibleOrder() {
        let first = makeEntry(changeId: "first", commitId: "first-commit", isDivergent: false)
        let middle = makeEntry(changeId: "middle", commitId: "middle-commit", isDivergent: false)
        let last = makeEntry(changeId: "last", commitId: "last-commit", isDivergent: false)
        let viewModel = makeViewModel(
            entries: [first, middle, last],
            selectedId: "last",
            selectedIds: ["last", "first"],
            contextTargetId: nil
        )

        XCTAssertEqual(viewModel.selectedRevisions, ["first", "last"])
        XCTAssertTrue(viewModel.isSelected(first.change))
        XCTAssertFalse(viewModel.isSelected(middle.change))
    }

    func testBatchActionAvailabilityFollowsSelectionTopology() {
        let base = makeEntry(changeId: "base", commitId: "base-commit", isDivergent: false)
        let left = makeEntry(
            changeId: "left",
            commitId: "left-commit",
            parents: ["base-commit"],
            isDivergent: false
        )
        let right = makeEntry(
            changeId: "right",
            commitId: "right-commit",
            parents: ["base-commit"],
            isDivergent: false
        )
        let child = makeEntry(
            changeId: "child",
            commitId: "child-commit",
            parents: ["left-commit"],
            isDivergent: false
        )

        let heads = makeViewModel(
            entries: [child, left, right, base],
            selectedId: "right",
            selectedIds: ["left", "right"],
            contextTargetId: nil
        )
        let linear = makeViewModel(
            entries: [child, left, base],
            selectedId: "child",
            selectedIds: ["child", "left", "base"],
            contextTargetId: nil
        )
        let gap = makeViewModel(
            entries: [child, right, left, base],
            selectedId: "child",
            selectedIds: ["child", "left"],
            contextTargetId: nil
        )
        let immutable = makeEntry(
            changeId: "immutable",
            commitId: "immutable-commit",
            isImmutable: true,
            isDivergent: false
        )
        let immutableSelection = makeViewModel(
            entries: [left, immutable],
            selectedId: "left",
            selectedIds: ["left", "immutable"],
            contextTargetId: nil
        )
        let single = makeViewModel(
            entries: [child, left, right, base],
            selectedId: "child",
            selectedIds: ["child"],
            contextTargetId: nil
        )

        XCTAssertTrue(heads.canMergeSelection)
        XCTAssertFalse(heads.canDiffSelection)
        XCTAssertTrue(heads.canAbandonSelection)
        XCTAssertTrue(heads.canRebaseSelection(onto: base.change))
        XCTAssertFalse(heads.canRebaseSelection(onto: child.change))
        XCTAssertFalse(heads.canSquashSelection)
        XCTAssertFalse(linear.canMergeSelection)
        XCTAssertTrue(linear.canDiffSelection)
        XCTAssertTrue(linear.canSquashSelection)
        XCTAssertFalse(gap.canDiffSelection)
        XCTAssertFalse(gap.canSquashSelection)
        XCTAssertFalse(immutableSelection.canAbandonSelection)
        XCTAssertFalse(immutableSelection.canRebaseSelection(onto: base.change))
        XCTAssertFalse(single.canMergeSelectedChange(with: left.change))
        XCTAssertTrue(single.canMergeSelectedChange(with: right.change))
    }

    func testClearsHoveredContextTarget() {
        let entry = makeEntry(changeId: "hovered", commitId: "hovered-commit", isDivergent: false)
        let viewModel = makeViewModel(entries: [entry], selectedId: "selected", contextTargetId: "hovered")

        XCTAssertNil(viewModel.nextContextTargetId(hovering: false, entry: entry))
    }

    func testCancelsMissingHoverTarget() {
        let entry = makeEntry(changeId: "present", commitId: "present-commit", isDivergent: false)
        let viewModel = makeViewModel(entries: [entry], selectedId: nil, contextTargetId: nil)

        XCTAssertTrue(viewModel.shouldCancelRebaseDrag(for: "missing-commit"))
        XCTAssertFalse(viewModel.shouldCancelRebaseDrag(for: "present-commit"))
        XCTAssertFalse(viewModel.shouldCancelRebaseDrag(for: nil))
    }

    func testDivergentSiblingsReturnsOtherCommitsWithSameChangeId() {
        let a = makeEntry(changeId: "same", commitId: "commit-a", isDivergent: true)
        let b = makeEntry(changeId: "same", commitId: "commit-b", isDivergent: true)
        let other = makeEntry(changeId: "other", commitId: "commit-c", isDivergent: false)
        let viewModel = makeViewModel(entries: [a, b, other], selectedId: nil, contextTargetId: nil)

        XCTAssertEqual(viewModel.divergentSiblings(of: a.change).map(\.commitId.id), ["commit-b"])
    }

    func testDivergentSiblingsEmptyForNonDivergentChange() {
        let solo = makeEntry(changeId: "solo", commitId: "commit-a", isDivergent: false)
        let viewModel = makeViewModel(entries: [solo], selectedId: nil, contextTargetId: nil)

        XCTAssertTrue(viewModel.divergentSiblings(of: solo.change).isEmpty)
    }

    func testDivergentSiblingLabelUsesShortCommitAndFirstLine() {
        let change = makeEntry(changeId: "x", commitId: "abcdef1234567890", isDivergent: true).change
        XCTAssertEqual(DAGView.divergentSiblingLabel(change), "abcdef12 — entry")
    }

    func testMovesSelectionForwardAndBack() {
        let first = makeEntry(changeId: "first", commitId: "first-commit", isDivergent: false)
        let second = makeEntry(changeId: "second", commitId: "second-commit", isDivergent: false)
        let viewModel = makeViewModel(entries: [first, second], selectedId: "first", contextTargetId: nil)

        XCTAssertEqual(viewModel.selectedChangeId(afterMovingBy: 1), "second")
        XCTAssertNil(viewModel.selectedChangeId(afterMovingBy: -1))
    }

    func testMovesSelectionAcrossDivergentRowsByCommitId() {
        let first = makeEntry(changeId: "same", commitId: "first-commit", isDivergent: true)
        let second = makeEntry(changeId: "same", commitId: "second-commit", isDivergent: true)
        let viewModel = makeViewModel(entries: [first, second], selectedId: "first-commit", contextTargetId: nil)

        XCTAssertEqual(viewModel.selectedChangeId(afterMovingBy: 1), "second-commit")
    }

    func testUsesListEndsWithoutSelection() {
        let first = makeEntry(changeId: "first", commitId: "first-commit", isDivergent: false)
        let second = makeEntry(changeId: "second", commitId: "second-commit", isDivergent: false)
        let viewModel = makeViewModel(entries: [first, second], selectedId: nil, contextTargetId: nil)

        XCTAssertEqual(viewModel.selectedChangeId(afterMovingBy: 1), "first")
        XCTAssertEqual(viewModel.selectedChangeId(afterMovingBy: -1), "second")
    }

    func testUsesCommitIdForDivergentSelection() {
        let entry = makeEntry(changeId: "change", commitId: "commit", isDivergent: true)
        let viewModel = makeViewModel(entries: [entry], selectedId: nil, contextTargetId: nil)

        XCTAssertEqual(viewModel.selectedRevision(for: "change"), "commit")
    }

    func testDisallowsSquashIntoImmutableParent() {
        let parent = makeEntry(
            changeId: "parent",
            commitId: "parent-commit",
            isImmutable: true,
            isDivergent: false
        )
        let child = makeEntry(
            changeId: "child",
            commitId: "child-commit",
            parents: ["parent-commit"],
            isDivergent: false
        )
        let viewModel = makeViewModel(entries: [child, parent], selectedId: nil, contextTargetId: nil)

        XCTAssertFalse(viewModel.canSquashIntoParent(child.change))
    }

    func testAllowsSquashWhenParentIsOutsideLoadedPage() {
        let child = makeEntry(
            changeId: "child",
            commitId: "child-commit",
            parents: ["unloaded-parent-commit"],
            isDivergent: false
        )
        let viewModel = makeViewModel(entries: [child], selectedId: nil, contextTargetId: nil)

        XCTAssertTrue(viewModel.canSquashIntoParent(child.change))
    }

    func testScrollIdUsesCommitIdForDivergentChange() {
        let entry = makeEntry(changeId: "change", commitId: "commit", isDivergent: true)
        let viewModel = makeViewModel(entries: [entry], selectedId: nil, contextTargetId: nil)

        XCTAssertEqual(viewModel.scrollId(for: "change"), "commit")
    }

    func testBuildsBookmarkDiffRequestFromBookmarkedSelectionAndTarget() {
        let base = makeEntry(changeId: "base", commitId: "base-commit", bookmarks: ["main"], isDivergent: false)
        let head = makeEntry(changeId: "head", commitId: "head-commit", bookmarks: ["feature"], isDivergent: false)
        let viewModel = makeViewModel(entries: [base, head], selectedId: "base", contextTargetId: nil)

        let request = viewModel.bookmarkDiffRequest(from: "base", to: head.change)

        XCTAssertEqual(request?.compareFromRev, "fork_point(\"main\" | \"feature\")")
        XCTAssertEqual(request?.display, CompareDisplay(title: "PR Diff", from: "main", to: "feature"))
    }

    func testSkipsBookmarkDiffRequestForTrunkTarget() {
        let base = makeEntry(changeId: "base", commitId: "base-commit", bookmarks: ["feature"], isDivergent: false)
        let head = makeEntry(changeId: "head", commitId: "head-commit", bookmarks: ["main"], isDivergent: false)
        let viewModel = makeViewModel(entries: [base, head], selectedId: "base", contextTargetId: nil)

        XCTAssertNil(viewModel.bookmarkDiffRequest(from: "base", to: head.change))
    }

    func testQuotesBookmarkRevsetSymbols() {
        XCTAssertEqual(RevsetExpressions.bookmarkEndpoint(name: "feature-x").rev, "\"feature-x\"")
        XCTAssertEqual(RevsetExpressions.bookmarkEndpoint(name: "feature\"x").rev, "\"feature\\\"x\"")
    }

    func testCompareDisplayPrefersBookmarks() {
        let base = makeEntry(changeId: "base-change", commitId: "base-commit", bookmarks: ["main"], isDivergent: false)
        let head = makeEntry(
            changeId: "head-change",
            commitId: "head-commit",
            bookmarks: ["bookmark-diff"],
            isDivergent: false
        )

        let display = RevsetExpressions.compareDisplay(
            from: "head-change",
            to: "base-change",
            changes: [base.change, head.change]
        )

        XCTAssertEqual(display, CompareDisplay(title: "Comparing", from: "bookmark-diff", to: "main"))
    }

    func testCompareDisplayHandlesComplexAndQuotedRevsets() {
        let display = RevsetExpressions.compareDisplay(
            from: "\"feature-x\"",
            to: "fork_point(\"main\" | \"feature-x\")",
            changes: []
        )

        XCTAssertEqual(display.from, "feature-x")
        XCTAssertEqual(display.to, "fork_point(\"main\" | \"feature-x\")")
    }

    func testCombinedDiffDisplaySummarizesSelectedRange() {
        let newest = makeEntry(
            changeId: "tzyrxtutkvwr",
            commitId: "newest-commit",
            isDivergent: false
        )
        let oldest = makeEntry(
            changeId: "uqnzmqnlabcd",
            commitId: "oldest-commit",
            isDivergent: false
        )

        let display = RevsetExpressions.combinedDiffDisplay(changes: [newest.change, oldest.change])

        XCTAssertEqual(
            display,
            CompareDisplay(title: "2 Changes Selected", from: "uqnzmqnl", to: "tzyrxtut")
        )
    }

    func testUsesJKNavigation() {
        XCTAssertEqual(
            DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "j", controlPressed: false),
            1
        )
        XCTAssertEqual(
            DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "k", controlPressed: false),
            -1
        )
    }

    func testUsesCtrlNPNavigation() {
        XCTAssertEqual(
            DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "n", controlPressed: true),
            1
        )
        XCTAssertEqual(
            DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "p", controlPressed: true),
            -1
        )
    }

    func testIgnoresPlainNPNavigation() {
        XCTAssertNil(DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "n", controlPressed: false))
        XCTAssertNil(DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "p", controlPressed: false))
    }

    private func makeViewModel(
        entries: [GraphEntry],
        selectedId: String?,
        selectedIds: [String] = [],
        contextTargetId: String?
    ) -> DAGViewModel {
        DAGViewModel(
            entries: entries,
            selectedId: selectedId,
            selectedIds: selectedIds,
            compareFromId: nil,
            contextTargetId: contextTargetId,
            rebaseDrag: nil,
            bookmarkDrag: nil,
            colorScheme: .light,
            layout: DAGLayout(entries: entries)
        )
    }

    private func makeEntry(
        changeId: String,
        commitId: String,
        parents: [String] = [],
        bookmarks: [String] = [],
        isImmutable: Bool = false,
        isDivergent: Bool
    ) -> GraphEntry {
        GraphEntry(
            change: mockChangeInfo(
                changeId: changeId,
                commitId: commitId,
                description: "entry",
                parents: parents,
                bookmarks: bookmarks,
                isImmutable: isImmutable,
                isDivergent: isDivergent
            ),
            edges: parents.map { GraphEdge(target: $0, edgeType: .direct) }
        )
    }
}
