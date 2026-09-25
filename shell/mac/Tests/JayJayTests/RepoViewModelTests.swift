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

    func testGraphGenerationAdvancesOnlyWhenTheGraphActuallyChanges() throws {
        let viewModel = try XCTUnwrap(viewModel)
        let graph = try viewModel.repo.logGraph(revset: "all()")
        viewModel.setGraph(graph)
        let generation = viewModel.graphGeneration

        viewModel.setGraph(graph)
        XCTAssertEqual(viewModel.graphGeneration, generation)

        try viewModel.repo.newChange(parent: "@", message: "another")
        try viewModel.setGraph(viewModel.repo.logGraph(revset: "all()"))
        XCTAssertGreaterThan(viewModel.graphGeneration, generation)
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

    func testCleanBoxClearsWhenWorkingCopyMovesToEmptyChange() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.applyWorkingCopy(changeId: "old", description: "Described summary\n\nDescribed details")

        viewModel.applyWorkingCopy(changeId: "new", description: "")

        XCTAssertEqual(viewModel.commitSummaryDraft, "")
        XCTAssertEqual(viewModel.commitDescriptionDraft, "")
    }

    func testNewChangeClearsCommitBox() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.commitSummaryDraft = "Previous summary"
        viewModel.commitDescriptionDraft = "Previous details"
        let previousSignal = viewModel.successActionSignal

        viewModel.newChange(parent: "@")

        try await waitUntil("the new change succeeds") { viewModel.successActionSignal != previousSignal }
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
        try viewModel.setGraph(viewModel.repo.logGraph(revset: "all()"))
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
        try viewModel.setGraph(viewModel.repo.logGraph(revset: "all()"))
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
        try viewModel.setGraph(viewModel.repo.logGraph(revset: "all()"))
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
        try viewModel.setGraph(viewModel.repo.logGraph(revset: "all()"))
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

        try await waitUntil("the squash finishes") {
            !viewModel.isRefreshingInFlight && viewModel.successActionSignal != 0
        }
        XCTAssertNil(viewModel.error)
        XCTAssertEqual(viewModel.selectedChange?.info.changeId.id, destinationChangeId)
        XCTAssertNotEqual(
            viewModel.selectedChange?.info.commitId.id,
            untouchedSibling.commitId.id,
            "selection must land on the squashed-into sibling, not the untouched one"
        )
    }

    func testEvologRestoreRunsOnceAtATimeAndReloadsTheEvolog() async throws {
        let repoPath = try XCTUnwrap(viewModel?.repoPath)
        viewModel = nil
        let file = URL(fileURLWithPath: repoPath).appendingPathComponent("restore.txt")
        try "v1\n".write(to: file, atomically: true, encoding: .utf8)
        _ = try runJj(["describe", "-m", "restore target"], in: repoPath)
        try "v2\n".write(to: file, atomically: true, encoding: .utf8)
        _ = try runJj(["st"], in: repoPath)

        viewModel = try RepoViewModel(path: repoPath)
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.showEvolog(rev: "@")
        try await waitUntil("the evolog loads") { viewModel.evologEntries != nil }
        let entries = try XCTUnwrap(viewModel.evologEntries)
        let v1 = try XCTUnwrap(entries.first {
            (try? viewModel.repo.fileContent(rev: $0.commitId.id, path: "restore.txt")) == "v1\n"
        })

        viewModel.restoreEvologVersion(v1.commitId.id)
        viewModel.restoreEvologVersion(v1.commitId.id)
        XCTAssertEqual(viewModel.info, "A version is already being restored")

        try await waitUntil("the evolog reloads with the restored version") {
            (viewModel.evologEntries?.count ?? 0) > entries.count
        }
        XCTAssertEqual(try String(contentsOf: file, encoding: .utf8), "v1\n")
        XCTAssertEqual(try runJj(["log", "--no-graph", "-r", "divergent()", "-T", "change_id"], in: repoPath), "")
    }

    func testSelectionDropRebasesEverySelectedChange() async throws {
        let repoPath = try XCTUnwrap(viewModel?.repoPath)
        viewModel = nil
        _ = try runJj(["describe", "-m", "base"], in: repoPath)
        _ = try runJj(["new", "-m", "first"], in: repoPath)
        _ = try runJj(["new", "-m", "second"], in: repoPath)
        _ = try runJj(["new", "-m", "destination", "subject(exact:base)"], in: repoPath)

        viewModel = try RepoViewModel(path: repoPath)
        let viewModel = try XCTUnwrap(viewModel)
        let find = { (subject: String) in
            try viewModel.repo.logGraph(revset: "all()").map(\.change).first {
                $0.description.trimmingCharacters(in: .whitespacesAndNewlines) == subject
            }
        }
        let first = try XCTUnwrap(find("first"))
        let second = try XCTUnwrap(find("second"))
        let destination = try XCTUnwrap(find("destination"))
        try viewModel.setGraph(viewModel.repo.logGraph(revset: "all()"))
        let request = DAGRebaseRequest(
            sourceRev: second.selectionRevision,
            sourceChangeId: second.changeId.id,
            sourceCommitId: second.commitId.id,
            sourceLabel: "2 changes",
            destRev: destination.selectionRevision,
            destChangeId: destination.changeId.id,
            destCommitId: destination.commitId.id,
            destLabel: "destination",
            selectionCommitIds: [second.commitId.id, first.commitId.id]
        )

        var message: String?
        viewModel.rebase(request: request, onSuccess: { _, feedback in message = feedback.message })
        try await waitUntil("the rebase finishes") { message != nil }

        XCTAssertEqual(message, "Rebased 2 changes onto destination.")
        let movedFirst = try XCTUnwrap(find("first"))
        XCTAssertEqual(movedFirst.parents, [destination.commitId.id])
        XCTAssertEqual(try XCTUnwrap(find("second")).parents, [movedFirst.commitId.id])
    }

    func testSelectionDropReportsConflictsOutsideTheDraggedChange() async throws {
        let repoPath = try XCTUnwrap(viewModel?.repoPath)
        viewModel = nil
        let file = URL(fileURLWithPath: repoPath).appendingPathComponent("shared.txt")
        try "base\n".write(to: file, atomically: true, encoding: .utf8)
        _ = try runJj(["describe", "-m", "base"], in: repoPath)
        _ = try runJj(["new", "-m", "conflicting"], in: repoPath)
        try "selected\n".write(to: file, atomically: true, encoding: .utf8)
        _ = try runJj(["new", "-m", "clean", "subject(exact:base)"], in: repoPath)
        _ = try runJj(["new", "-m", "destination", "subject(exact:base)"], in: repoPath)
        try "destination\n".write(to: file, atomically: true, encoding: .utf8)

        viewModel = try RepoViewModel(path: repoPath)
        let viewModel = try XCTUnwrap(viewModel)
        let clean = try viewModel.repo.showSummary(rev: "subject(exact:clean)").info
        let conflicting = try viewModel.repo.showSummary(rev: "subject(exact:conflicting)").info
        let destination = try viewModel.repo.showSummary(rev: "@").info
        try viewModel.setGraph(viewModel.repo.logGraph(revset: "all()"))
        let request = DAGRebaseRequest(
            sourceRev: clean.selectionRevision,
            sourceChangeId: clean.changeId.id,
            sourceCommitId: clean.commitId.id,
            sourceLabel: "2 changes",
            destRev: destination.selectionRevision,
            destChangeId: destination.changeId.id,
            destCommitId: destination.commitId.id,
            destLabel: "destination",
            selectionCommitIds: [clean.commitId.id, conflicting.commitId.id]
        )

        var message: String?
        viewModel.rebase(request: request, onSuccess: { _, feedback in message = feedback.message })
        try await waitUntil("the rebase finishes") { message != nil }

        XCTAssertFalse(try viewModel.repo.showSummary(rev: clean.changeId.id).info.hasConflict)
        XCTAssertTrue(try viewModel.repo.showSummary(rev: conflicting.changeId.id).info.hasConflict)
        XCTAssertEqual(message, "Rebased 2 changes onto destination. Conflicts need resolution.")
    }

    func testRebaseIsCancelledWhenADraggedChangeWasRewrittenMeanwhile() throws {
        let repoPath = try XCTUnwrap(viewModel?.repoPath)
        viewModel = nil
        _ = try runJj(["describe", "-m", "base"], in: repoPath)
        _ = try runJj(["new", "-m", "first"], in: repoPath)
        _ = try runJj(["new", "-m", "second"], in: repoPath)
        _ = try runJj(["new", "-m", "destination", "subject(exact:base)"], in: repoPath)

        viewModel = try RepoViewModel(path: repoPath)
        let viewModel = try XCTUnwrap(viewModel)
        let before = try viewModel.repo.logGraph(revset: "all()").map(\.change)
        let find = { (subject: String) in
            try XCTUnwrap(before.first { $0.description.trimmingCharacters(in: .whitespacesAndNewlines) == subject })
        }
        let (first, second, destination) = try (find("first"), find("second"), find("destination"))
        let request = DAGRebaseRequest(
            sourceRev: second.selectionRevision,
            sourceChangeId: second.changeId.id,
            sourceCommitId: second.commitId.id,
            sourceLabel: "2 changes",
            destRev: destination.selectionRevision,
            destChangeId: destination.changeId.id,
            destCommitId: destination.commitId.id,
            destLabel: "destination",
            selectionCommitIds: [second.commitId.id, first.commitId.id]
        )
        try viewModel.repo.describe(rev: first.changeId.id, message: "first, reworded")
        try viewModel.setGraph(viewModel.repo.logGraph(revset: "all()"))

        var failure: String?
        viewModel.rebase(request: request, onSuccess: { _, _ in XCTFail("stale rebase ran") }, onFailure: { _, message in failure = message })

        XCTAssertEqual(failure, "Rebase cancelled: the changes moved while confirming")
        XCTAssertEqual(try runJj(["log", "--no-graph", "-r", "parents(\(first.changeId.id))", "-T", "description"], in: repoPath), "base")
    }

    func testCombinedComparisonCannotReverse() throws {
        let viewModel = try XCTUnwrap(viewModel)
        viewModel.compareFromId = "roots"
        viewModel.compareToId = "heads"
        viewModel.compareDisplay = CompareDisplay(
            title: "2 Changes Selected",
            from: "oldest",
            to: "newest",
            isCombinedSelection: true
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
