@testable import JayJay
import JayJayCore
import XCTest

@MainActor
class RepoViewModelTestCase: XCTestCase {
    private var tempDirectory: URL?
    var viewModel: RepoViewModel?

    override func setUpWithError() throws {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-view-model-tests-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try initJjGitRepo(path: directory.path)
        let repo = try JayJayRepo.open(path: directory.path)
        tempDirectory = directory
        viewModel = RepoViewModel(
            path: directory.path,
            repo: repo,
            workingCopyIsLarge: false,
            configWarning: nil
        )
    }

    override func tearDownWithError() throws {
        viewModel = nil
        if let tempDirectory {
            try? FileManager.default.removeItem(at: tempDirectory)
        }
        tempDirectory = nil
    }

    func waitUntil(_ what: String, _ condition: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(30)
        while !condition() {
            if Date() >= deadline {
                XCTFail("timed out waiting until \(what)")
                throw CancellationError()
            }
            try await Task.sleep(for: .milliseconds(20))
        }
    }
}
