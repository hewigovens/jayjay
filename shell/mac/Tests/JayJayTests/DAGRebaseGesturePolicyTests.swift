@testable import JayJay
import JayJayCore
import XCTest

final class DAGRebaseGesturePolicyTests: XCTestCase {
    func testBeginsPress() {
        let action = DAGRebaseGesturePolicy.changeAction(
            entryIsImmutable: false,
            sourceCommitId: "source",
            rebaseDrag: nil,
            location: CGPoint(x: 10, y: 20)
        )

        XCTAssertEqual(action, .beginPress)
    }

    func testCancelsPressOnLargeMove() {
        let state = makeDragState(phase: .pressing, startLocation: .zero)

        let action = DAGRebaseGesturePolicy.changeAction(
            entryIsImmutable: false,
            sourceCommitId: "source",
            rebaseDrag: state,
            location: CGPoint(x: 20, y: 0)
        )

        XCTAssertEqual(action, .cancelPress)
    }

    func testStartsDragAfterArm() {
        let state = makeDragState(phase: .armed, startLocation: .zero)

        let action = DAGRebaseGesturePolicy.changeAction(
            entryIsImmutable: false,
            sourceCommitId: "source",
            rebaseDrag: state,
            location: CGPoint(x: 3, y: 0)
        )

        XCTAssertEqual(action, .beginDragging)
    }

    func testSelectsImmutableCommit() {
        let action = DAGRebaseGesturePolicy.endAction(
            entryIsImmutable: true,
            sourceCommitId: "immutable",
            rebaseDrag: nil,
            startLocation: .zero,
            location: CGPoint(x: 2, y: 1)
        )

        XCTAssertEqual(action, .select)
    }

    func testIgnoresImmutableCommitAfterDrag() {
        let action = DAGRebaseGesturePolicy.endAction(
            entryIsImmutable: true,
            sourceCommitId: "immutable",
            rebaseDrag: nil,
            startLocation: .zero,
            location: CGPoint(x: 20, y: 0)
        )

        XCTAssertEqual(action, .ignore)
    }

    func testSelectsMutableCommitBeforeArm() {
        let action = DAGRebaseGesturePolicy.endAction(
            entryIsImmutable: false,
            sourceCommitId: "source",
            rebaseDrag: makeDragState(phase: .pressing),
            startLocation: .zero,
            location: CGPoint(x: 1, y: 1)
        )

        XCTAssertEqual(action, .select)
    }

    func testConfirmsDrop() {
        let action = DAGRebaseGesturePolicy.endAction(
            entryIsImmutable: false,
            sourceCommitId: "source",
            rebaseDrag: makeDragState(phase: .dragging),
            startLocation: .zero,
            location: CGPoint(x: 12, y: 4)
        )

        XCTAssertEqual(action, .confirmDrop)
    }

    func testAllowsAncestorTarget() {
        let ancestor = makeEntry(
            changeId: "base-change",
            commitId: "base-commit",
            description: "main",
            isImmutable: false
        )
        let source = makeEntry(
            changeId: "source-change",
            commitId: "source-commit",
            description: "feat-x",
            isImmutable: false,
            parents: ["base-commit"]
        )

        let request = DAGRebaseGesturePolicy.dropRequest(
            rebaseDrag: makeDragState(phase: .dragging),
            previewTargetCommitId: nil,
            hoveredCommitId: "base-commit",
            entries: [source, ancestor]
        )

        XCTAssertEqual(request?.sourceChangeId, "change")
        XCTAssertEqual(request?.destCommitId, "base-commit")
        XCTAssertEqual(request?.destRev, "base-change")
        XCTAssertEqual(request?.destLabel, "main")
    }

    func testAllowsImmutableTarget() {
        let immutableTarget = makeEntry(
            changeId: "target-change",
            commitId: "target-commit",
            description: "",
            isImmutable: true,
            bookmarks: ["main"]
        )

        let request = DAGRebaseGesturePolicy.dropRequest(
            rebaseDrag: makeDragState(phase: .dragging),
            previewTargetCommitId: nil,
            hoveredCommitId: "target-commit",
            entries: [immutableTarget]
        )

        XCTAssertEqual(request?.destCommitId, "target-commit")
        XCTAssertEqual(request?.destRev, "target-change")
        XCTAssertEqual(request?.destLabel, "main")
    }

    private func makeDragState(
        phase: DAGRebasePhase,
        startLocation: CGPoint = .zero
    ) -> DAGRebaseDragState {
        DAGRebaseDragState(
            sourceCommitId: "source",
            sourceChangeId: "change",
            sourceRev: "change",
            sourceLabel: "feat-x",
            startLocation: startLocation,
            armedAt: phase == .pressing ? nil : Date(timeIntervalSinceReferenceDate: 10),
            phase: phase,
            location: startLocation,
            hoveredCommitId: nil
        )
    }

    private func makeEntry(
        changeId: String,
        commitId: String,
        description: String,
        isImmutable: Bool,
        parents: [String] = [],
        bookmarks: [String] = []
    ) -> GraphEntry {
        GraphEntry(
            change: mockChangeInfo(
                changeId: changeId,
                commitId: commitId,
                description: description,
                parents: parents,
                bookmarks: bookmarks,
                isImmutable: isImmutable
            ),
            edges: []
        )
    }
}

final class BookmarkDragGesturePolicyTests: XCTestCase {
    func testBeginsPressWithNoDrag() {
        let action = BookmarkDragGesturePolicy.changeAction(
            bookmarkName: "main", drag: nil, location: CGPoint(x: 5, y: 5)
        )
        XCTAssertEqual(action, .beginPress)
    }

    func testStartsDraggingFromPressOnMove() {
        // A bookmark chip starts dragging on movement, with no hold/arm required.
        let action = BookmarkDragGesturePolicy.changeAction(
            bookmarkName: "main", drag: makeDrag(phase: .pressing), location: CGPoint(x: 20, y: 0)
        )
        XCTAssertEqual(action, .beginDragging)
    }

    func testStartsDraggingAfterArm() {
        let action = BookmarkDragGesturePolicy.changeAction(
            bookmarkName: "main", drag: makeDrag(phase: .armed), location: CGPoint(x: 3, y: 0)
        )
        XCTAssertEqual(action, .beginDragging)
    }

    func testUpdatesWhileDragging() {
        let action = BookmarkDragGesturePolicy.changeAction(
            bookmarkName: "main", drag: makeDrag(phase: .dragging), location: CGPoint(x: 50, y: 50)
        )
        XCTAssertEqual(action, .updateDragging)
    }

    func testEndWhilePressingCancels() {
        XCTAssertEqual(
            BookmarkDragGesturePolicy.endAction(bookmarkName: "main", drag: makeDrag(phase: .pressing)),
            .cancel
        )
    }

    func testEndWhileDraggingConfirms() {
        XCTAssertEqual(
            BookmarkDragGesturePolicy.endAction(bookmarkName: "main", drag: makeDrag(phase: .dragging)),
            .confirmDrop
        )
    }

    func testDropRequestResolvesTarget() {
        let target = makeEntry(changeId: "target-change", commitId: "target-commit")
        let request = BookmarkDragGesturePolicy.dropRequest(
            drag: makeDrag(phase: .dragging),
            previewTargetCommitId: nil,
            hoveredCommitId: "target-commit",
            entries: [target]
        )
        XCTAssertEqual(request?.bookmarkName, "main")
        XCTAssertEqual(request?.destCommitId, "target-commit")
        XCTAssertEqual(request?.destRev, "target-change")
    }

    func testDropRequestRejectsSelfDrop() {
        let source = makeEntry(changeId: "source-change", commitId: "source-commit")
        let request = BookmarkDragGesturePolicy.dropRequest(
            drag: makeDrag(phase: .dragging, sourceCommitId: "source-commit"),
            previewTargetCommitId: nil,
            hoveredCommitId: "source-commit",
            entries: [source]
        )
        XCTAssertNil(request)
    }

    private func makeDrag(
        phase: DAGRebasePhase,
        sourceCommitId: String = "source-commit"
    ) -> BookmarkDragState {
        BookmarkDragState(
            bookmarkName: "main",
            sourceCommitId: sourceCommitId,
            startLocation: .zero,
            armedAt: phase == .pressing ? nil : Date(timeIntervalSinceReferenceDate: 10),
            phase: phase,
            location: .zero,
            hoveredCommitId: nil
        )
    }

    private func makeEntry(changeId: String, commitId: String) -> GraphEntry {
        GraphEntry(
            change: mockChangeInfo(changeId: changeId, commitId: commitId, description: "entry"),
            edges: []
        )
    }
}
