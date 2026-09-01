@testable import JayJay
import JayJayCore
import XCTest

final class EvologViewModelTests: XCTestCase {
    func testSelectingACollapsedRunUsesItsNewestSnapshot() {
        let viewModel = makeSnapshotViewModel()
        viewModel.select(EvologRow(start: 1, count: 12), click: .replace)
        XCTAssertEqual(viewModel.selectedIndex, 1)
        XCTAssertEqual(viewModel.selectedFromCommitId, "c1")
        XCTAssertEqual(viewModel.displayedRows.count, 14)

        viewModel.setHideSnapshots(false)
        viewModel.setHideSnapshots(true)
        XCTAssertEqual(viewModel.displayedRows.count, 3, "hiding again collapses the expanded run")
    }

    func testHidingRetargetsAMiddleSnapshotSelectionToTheRunNewest() {
        let viewModel = makeSnapshotViewModel()
        viewModel.setHideSnapshots(false)
        viewModel.select(EvologRow(start: 7, count: 1), click: .replace)
        viewModel.setHideSnapshots(true)
        XCTAssertEqual(viewModel.selectedIndex, 1)
        XCTAssertEqual(viewModel.selectedFromCommitId, "c1")
    }

    func testSelectedVersionsDefineInterdiffEndpoints() {
        let entries = (0 ... 3).map { index in
            EvologEntry(
                changeId: ShortId(id: "change", shortLen: 1),
                commitId: ShortId(id: "commit-\(index)", shortLen: 1),
                timestampMillis: 0,
                operation: "rewrite",
                description: "version \(index)"
            )
        }
        let viewModel = EvologViewModel(
            entries: entries,
            changeId: "change",
            repo: nil,
            diffStore: DiffStore()
        )

        viewModel.select(EvologRow(start: 2, count: 1), click: .replace)
        XCTAssertEqual(viewModel.selectedFromCommitId, "commit-2")
        XCTAssertEqual(viewModel.selectedToCommitId, "commit-0")

        viewModel.select(EvologRow(start: 1, count: 1), click: .toggle)
        XCTAssertEqual(viewModel.selectedFromCommitId, "commit-2")
        XCTAssertEqual(viewModel.selectedToCommitId, "commit-1")

        viewModel.reverseComparison()
        XCTAssertTrue(viewModel.comparisonReversed)
        XCTAssertEqual(viewModel.selectedFromCommitId, "commit-1")
        XCTAssertEqual(viewModel.selectedToCommitId, "commit-2")

        viewModel.select(EvologRow(start: 3, count: 1), click: .extend)
        XCTAssertFalse(viewModel.comparisonReversed)
        XCTAssertEqual(viewModel.selection.orderedIDs(in: Array(entries.indices)), [2, 3])
        XCTAssertEqual(viewModel.selectedFromCommitId, "commit-3")
        XCTAssertEqual(viewModel.selectedToCommitId, "commit-2")
    }

    func testInterdiffSummaryFailureIsVisible() async throws {
        let (directory, repo) = try makeRepo()
        defer { try? FileManager.default.removeItem(at: directory) }
        let viewModel = EvologViewModel(
            entries: [makeEntry("missing-new"), makeEntry("missing-old")],
            changeId: "change",
            repo: repo,
            diffStore: DiffStore()
        )

        viewModel.select(EvologRow(start: 1, count: 1), click: .replace)
        for _ in 0 ..< 100 where viewModel.interdiffError == nil {
            try await Task.sleep(for: .milliseconds(10))
        }

        XCTAssertNotNil(viewModel.interdiffError)
        XCTAssertFalse(viewModel.interdiffLoading)
    }

    func testInterdiffFileFailureIsVisible() async throws {
        let (directory, repo) = try makeRepo()
        defer { try? FileManager.default.removeItem(at: directory) }
        try repo.newChange(parent: "@", message: "new version")
        let changes = try repo.log(revset: "all()")
        XCTAssertGreaterThanOrEqual(changes.count, 2)
        guard changes.count >= 2 else { return }
        let viewModel = EvologViewModel(
            entries: changes.prefix(2).map {
                EvologEntry(
                    changeId: $0.changeId,
                    commitId: $0.commitId,
                    timestampMillis: 0,
                    operation: "rewrite",
                    description: $0.description
                )
            },
            changeId: changes[0].changeId.id,
            repo: repo,
            diffStore: DiffStore()
        )
        viewModel.select(EvologRow(start: 1, count: 1), click: .replace)
        for _ in 0 ..< 100 where viewModel.interdiffDetail == nil && viewModel.interdiffError == nil {
            try await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertNil(viewModel.interdiffError)
        XCTAssertNotNil(viewModel.interdiffDetail)

        viewModel.selectedPath = "missing.txt"
        viewModel.loadFile(path: "missing.txt")
        for _ in 0 ..< 100 where viewModel.fileError == nil {
            try await Task.sleep(for: .milliseconds(10))
        }

        XCTAssertNotNil(viewModel.fileError)
        XCTAssertFalse(viewModel.fileLoading)
    }
}

private func makeEntry(_ commitId: String) -> EvologEntry {
    EvologEntry(
        changeId: ShortId(id: "change", shortLen: 1),
        commitId: ShortId(id: commitId, shortLen: 1),
        timestampMillis: 0,
        operation: "rewrite",
        description: "version"
    )
}

private func makeRepo() throws -> (URL, JayJayRepo) {
    let directory = FileManager.default.temporaryDirectory
        .appending(path: "jayjay-evolog-error-tests-\(UUID().uuidString)")
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    do {
        try initJjGitRepo(path: directory.path)
        return try (directory, JayJayRepo.open(path: directory.path))
    } catch {
        try? FileManager.default.removeItem(at: directory)
        throw error
    }
}

private func makeSnapshotViewModel() -> EvologViewModel {
    let operations = ["squash commits abc"]
        + Array(repeating: "snapshot working copy", count: 12)
        + ["describe commit def"]
    let entries = operations.enumerated().map { index, operation in
        EvologEntry(
            changeId: ShortId(id: "change", shortLen: 1),
            commitId: ShortId(id: "c\(index)", shortLen: 1),
            timestampMillis: Int64(index),
            operation: operation,
            description: ""
        )
    }
    return EvologViewModel(entries: entries, changeId: "change", repo: nil, diffStore: DiffStore())
}
