@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class ScriptRepositoryTests: RepoViewModelTestCase {
    func testSelectionExportsChangeIdentityInsteadOfRevisionExpressions() throws {
        let model = try XCTUnwrap(viewModel)
        let repository = ScriptRepository(viewModel: model)
        let first = mockChangeInfo(changeId: "first", commitId: "commit-a", isDivergent: true)
        let second = mockChangeInfo(changeId: "second", commitId: "commit-b")
        model.setGraph([first, second].map { GraphEntry(change: $0, edges: []) })
        model.selectedChangeIds = [second.selectionRevision, first.selectionRevision, first.changeId.id]
        XCTAssertEqual(repository.selectedChangeIDs, ["second", "first"])

        model.setGraph([])
        let detail = try model.repo.show(rev: "@")
        model.applySingleSelectedChange(detail)
        XCTAssertEqual(repository.selectedChangeIDs, [detail.info.changeId.id])

        model.selectedChangeIds = []
        XCTAssertEqual(repository.selectedChangeIDs, [])
    }

    func testDeadReferenceFailsInsteadOfReturningAnEmptySelection() throws {
        let model = try XCTUnwrap(viewModel)
        let repository = ScriptRepository(viewModel: model)

        model.switchTarget = "/elsewhere"
        XCTAssertThrowsError(try repository.exportedSelectedChangeIDs()) { error in
            XCTAssertEqual((error as? ScriptError)?.number, -1728)
        }

        model.switchTarget = nil
        model.beginShutdown()
        XCTAssertThrowsError(try repository.exportedSelectedChangeIDs()) { error in
            XCTAssertEqual((error as? ScriptError)?.number, -1728)
        }
    }

    func testSettlingFailsWhenTheRefreshFoundTheWorkspaceGone() async throws {
        let model = try XCTUnwrap(viewModel)
        model.workspaceVanished = true
        do {
            try await ScriptRepository.settle(model)
            XCTFail("A vanished workspace settled as a success")
        } catch {
            XCTAssertEqual((error as? ScriptError)?.number, -10000)
        }
    }
}
