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
            primaryRoot: directory.path,
            workingCopyIsLarge: false,
            defaultRevsetPreset: repo.defaultRevsetPreset()
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

    @discardableResult
    func runJj(_ arguments: [String], in repoPath: String) throws -> String {
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
