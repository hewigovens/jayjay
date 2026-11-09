@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class OverviewViewModelTests: XCTestCase {
    func testFailedChangeFileLoadIsNotShownAsNoChanges() async throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-overview-tests-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        try initJjGitRepo(path: directory.path)
        let viewModel = OverviewViewModel(repoPath: directory.path)
        await viewModel.open()
        defer { viewModel.close() }
        viewModel.selectedChangeId = "no-such-revision"

        await viewModel.loadSelectedChangeFiles()

        guard case .failure = viewModel.selectedChangeFiles else {
            return XCTFail("a failed stats load must not read as \"no file changes\"")
        }
    }
}
