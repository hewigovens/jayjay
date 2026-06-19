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
        contextTargetId: String?
    ) -> DAGViewModel {
        DAGViewModel(
            entries: entries,
            selectedId: selectedId,
            compareFromId: nil,
            contextTargetId: contextTargetId,
            rebaseDrag: nil,
            colorScheme: .light,
            layout: DAGLayout(entries: entries)
        )
    }

    private func makeEntry(
        changeId: String,
        commitId: String,
        bookmarks: [String] = [],
        isDivergent: Bool
    ) -> GraphEntry {
        GraphEntry(
            change: ChangeInfo(
                changeId: changeId,
                changeIdShortLen: 1,
                commitId: commitId,
                commitIdShortLen: 1,
                description: "entry",
                author: .tester,
                parents: [],
                bookmarks: bookmarks,
                tags: [],
                isWorkingCopy: false,
                hasConflict: false,
                isEmpty: false,
                isImmutable: false,
                isDivergent: isDivergent
            ),
            edges: []
        )
    }
}
