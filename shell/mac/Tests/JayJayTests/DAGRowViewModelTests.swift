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
            colorScheme: .dark
        )

        XCTAssertTrue(viewModel.isRebaseCandidate)
        XCTAssertTrue(viewModel.isRebaseHoverTarget)
        XCTAssertEqual(viewModel.dragTargetText, "Rebase feat-x onto main?")
        XCTAssertTrue(viewModel.showsReturnHint)
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
                changeId: changeId,
                commitId: commitId,
                description: description,
                author: .tester,
                parents: [],
                bookmarks: [],
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
}
