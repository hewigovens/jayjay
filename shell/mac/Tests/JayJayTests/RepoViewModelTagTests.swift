@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoViewModelTagTests: RepoViewModelTestCase {
    func testDeleteAndPushRemovesRemoteTagAndRespectsPushGate() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let remote = FileManager.default.temporaryDirectory.appending(path: UUID().uuidString)
        defer { try? FileManager.default.removeItem(at: remote) }
        try git(["init", "--bare", remote.path], at: viewModel.repoPath)
        try git(["remote", "add", "origin", remote.path], at: viewModel.repoPath)
        try viewModel.repo.createTag(name: "v1.0", rev: "@")
        _ = try viewModel.repo.gitPushTag(tag: "v1.0", sync: viewModel.repo.syncToken())
        XCTAssertEqual(try git(["tag", "--list"], at: remote.path), "v1.0")

        viewModel.isPushingInFlight = true
        viewModel.deleteTagAndPush(name: "v1.0")
        XCTAssertEqual(viewModel.info, "Push already in progress")
        XCTAssertEqual(try git(["tag", "--list"], at: viewModel.repoPath), "v1.0")
        viewModel.isPushingInFlight = false

        try git(["remote", "set-url", "origin", remote.appending(path: "missing.git").path], at: viewModel.repoPath)
        viewModel.deleteTagAndPush(name: "v1.0")
        try await waitUntil("failed push releases gate") {
            viewModel.error != nil && !viewModel.isPushingInFlight
        }
        XCTAssertEqual(try git(["tag", "--list"], at: viewModel.repoPath), "v1.0")
        try git(["remote", "set-url", "origin", remote.path], at: viewModel.repoPath)

        viewModel.deleteTagAndPush(name: "v1.0")
        try await waitUntil("remote tag deletion") {
            (try? self.git(["tag", "--list"], at: remote.path)) == ""
                && !viewModel.isPushingInFlight
        }
        XCTAssertNil(viewModel.error)
        XCTAssertEqual(try git(["tag", "--list"], at: viewModel.repoPath), "")
    }

    @discardableResult
    private func git(_ arguments: [String], at path: String) throws -> String {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/git")
        process.arguments = ["-C", path] + arguments
        let output = Pipe()
        process.standardOutput = output
        try process.run()
        let data = output.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()
        XCTAssertEqual(process.terminationStatus, 0)
        return String(decoding: data, as: UTF8.self).trimmingCharacters(in: .whitespacesAndNewlines)
    }
}
