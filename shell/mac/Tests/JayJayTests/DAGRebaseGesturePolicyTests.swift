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
            hoveredPlacement: .onto,
            entries: [source, ancestor]
        )

        XCTAssertEqual(request?.sourceChangeId, "change")
        XCTAssertEqual(request?.destCommitId, "base-commit")
        XCTAssertEqual(request?.destRev, "base-change")
        XCTAssertEqual(request?.destLabel, "main")
        XCTAssertEqual(request?.placement, .onto)
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
            hoveredPlacement: .after,
            entries: [immutableTarget]
        )

        XCTAssertEqual(request?.destCommitId, "target-commit")
        XCTAssertEqual(request?.destRev, "target-change")
        XCTAssertEqual(request?.destLabel, "main")
        XCTAssertEqual(request?.placement, .after)
    }

    func testRejectsInsertBeforeImmutableTarget() {
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
            hoveredPlacement: .before,
            entries: [immutableTarget]
        )

        XCTAssertNil(request)
    }

    func testPlacementZones() {
        let frame = CGRect(x: 0, y: 10, width: 120, height: 90)

        XCTAssertEqual(DAGRebaseGesturePolicy.placement(location: CGPoint(x: 20, y: 20), rowFrame: frame), .before)
        XCTAssertEqual(DAGRebaseGesturePolicy.placement(location: CGPoint(x: 20, y: 55), rowFrame: frame), .onto)
        XCTAssertEqual(DAGRebaseGesturePolicy.placement(location: CGPoint(x: 20, y: 95), rowFrame: frame), .after)
    }

    func testValidPlacementRejectsImmutableTopBand() {
        let frame = CGRect(x: 0, y: 10, width: 120, height: 90)

        XCTAssertNil(DAGRebaseGesturePolicy.validPlacement(
            location: CGPoint(x: 20, y: 20),
            rowFrame: frame,
            targetIsImmutable: true
        ))
        XCTAssertEqual(DAGRebaseGesturePolicy.validPlacement(
            location: CGPoint(x: 20, y: 55),
            rowFrame: frame,
            targetIsImmutable: true
        ), .onto)
        XCTAssertEqual(DAGRebaseGesturePolicy.validPlacement(
            location: CGPoint(x: 20, y: 95),
            rowFrame: frame,
            targetIsImmutable: true
        ), .after)
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
            hoveredCommitId: nil,
            hoveredPlacement: nil
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
            change: ChangeInfo(
                changeId: changeId,
                commitId: commitId,
                description: description,
                author: "Tester",
                email: "tester@example.com",
                timestampMillis: 0,
                parents: parents,
                bookmarks: bookmarks,
                isWorkingCopy: false,
                hasConflict: false,
                isEmpty: false,
                isImmutable: isImmutable,
                isDivergent: false
            ),
            edges: []
        )
    }
}
