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

    func testUsesJKNavigation() {
        XCTAssertEqual(DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "j", controlPressed: false), 1)
        XCTAssertEqual(DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "k", controlPressed: false), -1)
    }

    func testUsesCtrlNPNavigation() {
        XCTAssertEqual(DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "n", controlPressed: true), 1)
        XCTAssertEqual(DAGViewModel.selectionDelta(keyCode: 0, charactersIgnoringModifiers: "p", controlPressed: true), -1)
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

    private func makeEntry(changeId: String, commitId: String, isDivergent: Bool) -> GraphEntry {
        GraphEntry(
            change: ChangeInfo(
                changeId: changeId,
                commitId: commitId,
                description: "entry",
                author: "Tester",
                email: "tester@example.com",
                timestampMillis: 0,
                parents: [],
                bookmarks: [],
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
