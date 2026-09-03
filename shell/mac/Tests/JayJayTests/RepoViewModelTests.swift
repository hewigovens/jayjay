@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoViewModelTests: RepoViewModelTestCase {
    func testApplyingSingleSelectionClearsMultiSelectionAndComparison() throws {
        let viewModel = try XCTUnwrap(viewModel)
        let detail = try viewModel.repo.showSummary(rev: "@")
        viewModel.selectedChangeIds = ["first", "second"]
        viewModel.compareFromId = "first"
        viewModel.compareToId = "second"

        viewModel.applySingleSelectedChange(detail)

        XCTAssertEqual(viewModel.selectedChangeIds, [detail.info.selectionRevision])
        XCTAssertNil(viewModel.compareFromId)
        XCTAssertNil(viewModel.compareToId)
    }

    func testDraftSurvivesMoveToEmptyWorkingCopy() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.applyWorkingCopy(changeId: "old", description: "")
        viewModel.commitSummaryDraft = "Typed summary"
        viewModel.commitDescriptionDraft = "Typed details"

        viewModel.applyWorkingCopy(changeId: "new", description: "")

        XCTAssertEqual(viewModel.commitSummaryDraft, "Typed summary")
        XCTAssertEqual(viewModel.commitDescriptionDraft, "Typed details")
    }

    func testTypedDraftSurvivesMoveToDescribedWorkingCopy() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.applyWorkingCopy(changeId: "old", description: "")
        viewModel.commitSummaryDraft = "Typed summary"
        viewModel.commitDescriptionDraft = "Typed details"

        viewModel.applyWorkingCopy(
            changeId: "new",
            description: "Incoming summary\n\nIncoming details"
        )

        XCTAssertEqual(viewModel.commitSummaryDraft, "Typed summary")
        XCTAssertEqual(viewModel.commitDescriptionDraft, "Typed details")
    }

    func testCleanBoxFollowsDescribedWorkingCopy() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.applyWorkingCopy(changeId: "old", description: "")

        viewModel.applyWorkingCopy(
            changeId: "new",
            description: "Incoming summary\n\nIncoming details"
        )

        XCTAssertEqual(viewModel.commitSummaryDraft, "Incoming summary")
        XCTAssertEqual(viewModel.commitDescriptionDraft, "Incoming details")
    }

    func testCleanBoxFollowsExternalDescriptionChange() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.applyWorkingCopy(changeId: "same", description: "Original")

        viewModel.applyWorkingCopy(
            changeId: "same",
            description: "Updated summary\n\nUpdated details"
        )

        XCTAssertEqual(viewModel.commitSummaryDraft, "Updated summary")
        XCTAssertEqual(viewModel.commitDescriptionDraft, "Updated details")
    }

    func testNewChangeClearsCommitBox() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.commitSummaryDraft = "Previous summary"
        viewModel.commitDescriptionDraft = "Previous details"
        let previousSignal = viewModel.successActionSignal

        viewModel.newChange(parent: "@")

        for _ in 0 ..< 100 where viewModel.successActionSignal == previousSignal {
            try await Task.sleep(for: .milliseconds(20))
        }
        XCTAssertGreaterThan(viewModel.successActionSignal, previousSignal)
        XCTAssertEqual(viewModel.commitSummaryDraft, "")
        XCTAssertEqual(viewModel.commitDescriptionDraft, "")
    }

    func testKeyboardSelectionLoadsTheChangeTheKeySettlesOn() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        try viewModel.applySingleSelectedChange(viewModel.repo.showSummary(rev: "@"))
        viewModel.select(changeId: "root()", coalescing: true)
        viewModel.select(changeId: "@", coalescing: true)
        XCTAssertEqual(viewModel.selectedChangeId, "@")
        XCTAssertNotNil(viewModel.selectedChange, "keyboard navigation should retain the current detail while coalescing")

        for _ in 0 ..< 200 where viewModel.selectedChangeId == "@" {
            try await Task.sleep(for: .milliseconds(20))
        }
        let detail = try XCTUnwrap(viewModel.selectedChange)
        XCTAssertTrue(detail.info.isWorkingCopy, "the earlier root() load must not win over the settled selection")
        XCTAssertEqual(viewModel.selectedChangeId, detail.info.selectionRevision)
    }

    func testNormalSelectionRetainsDetailWhileLoading() throws {
        let viewModel = try XCTUnwrap(viewModel)
        try viewModel.applySingleSelectedChange(viewModel.repo.showSummary(rev: "@"))

        viewModel.select(changeId: "root()", coalescing: false)

        XCTAssertNotNil(viewModel.selectedChange)
    }

    func testNonConsecutiveSelectionComparesOutermostChanges() throws {
        let viewModel = try XCTUnwrap(viewModel)
        try viewModel.repo.newChange(parent: "@", message: "middle")
        try viewModel.repo.newChange(parent: "@", message: "newest")
        viewModel.graphEntries = try viewModel.repo.logGraph(revset: "all()")
        XCTAssertGreaterThanOrEqual(viewModel.changes.count, 3)
        guard viewModel.changes.count >= 3 else { return }

        let newest = viewModel.changes[0]
        let oldest = viewModel.changes[2]
        let first = newest.selectionRevision
        let third = oldest.selectionRevision
        viewModel.selectedChangeId = first
        viewModel.selectedChangeIds = [first]
        viewModel.evologRev = first
        viewModel.evologEntries = []
        viewModel.prInfo = PrInfo(
            number: 7,
            state: .open,
            title: "Previous change",
            url: "https://example.com/pr/7",
            checks: .none
        )
        let prFetchTask = Task<Void, Never> { _ = try? await Task.sleep(for: .seconds(30)) }
        viewModel.prFetchTask = prFetchTask

        viewModel.updateSelection(changeId: third, click: .toggle)

        XCTAssertEqual(viewModel.selectedChangeIds, [first, third])
        XCTAssertEqual(viewModel.selectedChangeId, third)
        XCTAssertEqual(viewModel.compareFromId, oldest.commitId.id)
        XCTAssertEqual(viewModel.compareToId, newest.commitId.id)
        XCTAssertEqual(viewModel.compareDisplay?.title, "Comparing")
        XCTAssertTrue(viewModel.canReverseCompare)
        XCTAssertNil(viewModel.evologRev)
        XCTAssertNil(viewModel.evologEntries)
        XCTAssertNil(viewModel.prInfo)
        XCTAssertNil(viewModel.prFetchTask)
        XCTAssertTrue(prFetchTask.isCancelled)
    }

    func testRemovingNonPrimarySelectionPreservesPrimary() throws {
        let viewModel = try XCTUnwrap(viewModel)
        try viewModel.repo.newChange(parent: "@", message: "middle")
        try viewModel.repo.newChange(parent: "@", message: "newest")
        viewModel.graphEntries = try viewModel.repo.logGraph(revset: "all()")
        XCTAssertGreaterThanOrEqual(viewModel.changes.count, 3)
        guard viewModel.changes.count >= 3 else { return }
        let selected = viewModel.changes.prefix(3).map(\.selectionRevision)
        viewModel.selectedChangeIds = selected
        viewModel.selectedChangeId = selected[2]

        viewModel.updateSelection(changeId: selected[1], click: .toggle)

        XCTAssertEqual(viewModel.selectedChangeIds, [selected[0], selected[2]])
        XCTAssertEqual(viewModel.selectedChangeId, selected[2])
    }

    func testRangeSelectionKeepsItsAnchorAcrossRepeatedExtensions() throws {
        let viewModel = try XCTUnwrap(viewModel)
        try viewModel.repo.newChange(parent: "@", message: "middle")
        try viewModel.repo.newChange(parent: "@", message: "newest")
        viewModel.graphEntries = try viewModel.repo.logGraph(revset: "all()")
        XCTAssertGreaterThanOrEqual(viewModel.changes.count, 3)
        guard viewModel.changes.count >= 3 else { return }
        let revisions = viewModel.changes.prefix(3).map(\.selectionRevision)
        viewModel.selectedChangeId = revisions[2]
        viewModel.selectedChangeIds = [revisions[2]]
        viewModel.selectedChangeAnchorId = revisions[2]

        viewModel.updateSelection(changeId: revisions[0], click: .extend)
        viewModel.updateSelection(changeId: revisions[1], click: .extend)

        XCTAssertEqual(viewModel.selectedChangeIds, [revisions[1], revisions[2]])
        XCTAssertEqual(viewModel.selectedChangeId, revisions[1])
        XCTAssertEqual(viewModel.selectedChangeAnchorId, revisions[2])
    }

    func testBatchSquashRetainsDivergentDestinationSelection() async throws {
        let repoPath = try XCTUnwrap(viewModel?.repoPath)
        viewModel = nil
        let baseOp = try runJj(
            ["op", "log", "--no-graph", "--limit", "1", "-T", "id"],
            in: repoPath
        )
        _ = try runJj(["describe", "-m", "oldest left"], in: repoPath)
        _ = try runJj(
            ["--at-op", baseOp, "describe", "-m", "oldest right"],
            in: repoPath
        )
        _ = try runJj(["new", "-m", "newest", "@"], in: repoPath)

        viewModel = try RepoViewModel(path: repoPath)
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.graphEntries = try viewModel.repo.logGraph(revset: "all()")
        let newest = try XCTUnwrap(
            viewModel.changes.first {
                $0.description.trimmingCharacters(in: .whitespacesAndNewlines) == "newest"
            }
        )
        let destination = try XCTUnwrap(
            viewModel.changes.first { newest.parents.contains($0.commitId.id) }
        )
        XCTAssertTrue(destination.isDivergent)
        let destinationChangeId = destination.changeId.id
        let untouchedSibling = try XCTUnwrap(
            viewModel.changes.first {
                $0.changeId.id == destinationChangeId && $0.commitId.id != destination.commitId.id
            }
        )

        viewModel.squash(revs: [newest.selectionRevision, destination.selectionRevision])

        for _ in 0 ..< 300 where viewModel.isRefreshingInFlight || viewModel.successActionSignal == 0 {
            try await Task.sleep(for: .milliseconds(20))
        }
        XCTAssertNil(viewModel.error)
        XCTAssertEqual(viewModel.selectedChange?.info.changeId.id, destinationChangeId)
        XCTAssertNotEqual(
            viewModel.selectedChange?.info.commitId.id,
            untouchedSibling.commitId.id,
            "selection must land on the squashed-into sibling, not the untouched one"
        )
    }

    func testCombinedComparisonCannotReverse() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.compareFromId = "roots"
        viewModel.compareToId = "heads"
        viewModel.compareDisplay = CompareDisplay(
            title: "2 Changes Selected",
            from: "oldest",
            to: "newest"
        )
        viewModel.selectedChangeIds = ["newest", "oldest"]

        XCTAssertFalse(viewModel.canReverseCompare)
        viewModel.reverseCompare()

        XCTAssertEqual(viewModel.compareFromId, "roots")
        XCTAssertEqual(viewModel.compareToId, "heads")
        XCTAssertEqual(viewModel.compareDisplay?.from, "oldest")
        XCTAssertEqual(viewModel.compareDisplay?.to, "newest")
    }

    private func runJj(_ arguments: [String], in repoPath: String) throws -> String {
        let process = Process()
        process.executableURL = try URL(fileURLWithPath: XCTUnwrap(findBinary(name: "jj")))
        process.arguments = ["-R", repoPath] + arguments
        let stdout = Pipe()
        process.standardOutput = stdout
        process.standardError = Pipe()
        try process.run()
        process.waitUntilExit()
        XCTAssertEqual(process.terminationStatus, 0, "jj \(arguments.joined(separator: " ")) failed")
        return String(decoding: stdout.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
            .trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
