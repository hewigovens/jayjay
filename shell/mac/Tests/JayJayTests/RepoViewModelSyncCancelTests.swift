@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepoViewModelSyncCancelTests: RepoViewModelTestCase {
    func testCancelingAPullReportsCancellationAndReleasesTheGate() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let scratch = URL(fileURLWithPath: viewModel.repoPath).deletingLastPathComponent()
        let gitStarted = scratch.appendingPathComponent("git-started-\(UUID().uuidString)")
        let fakeGit = scratch.appendingPathComponent("fake-git-\(UUID().uuidString)")
        try "#!/bin/sh\ntouch '\(gitStarted.path)'\nsleep 30\n".write(to: fakeGit, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: fakeGit.path)
        defer {
            try? FileManager.default.removeItem(at: fakeGit)
            try? FileManager.default.removeItem(at: gitStarted)
        }
        try runJj(["git", "remote", "add", "origin", "https://example.invalid/repo.git"], in: viewModel.repoPath)
        try runJj(["config", "set", "--repo", "git.executable-path", fakeGit.path], in: viewModel.repoPath)

        viewModel.gitFetch()
        XCTAssertTrue(viewModel.isPullingInFlight)
        try await waitUntil("fetch reaches git") { FileManager.default.fileExists(atPath: gitStarted.path) }

        viewModel.cancelPull()

        try await waitUntil("pull gate releases") { !viewModel.isPullingInFlight }
        XCTAssertEqual(viewModel.info, "Pull canceled")
        XCTAssertNil(viewModel.error)
        XCTAssertNotNil(viewModel.refreshTask, "a canceled pull still refreshes: its remote phase may have landed")
    }

    private func runJj(_ arguments: [String], in repoPath: String) throws {
        let process = Process()
        process.executableURL = try URL(fileURLWithPath: XCTUnwrap(findBinary(name: "jj")))
        process.arguments = ["-R", repoPath] + arguments
        try process.run()
        process.waitUntilExit()
        XCTAssertEqual(process.terminationStatus, 0, "jj \(arguments.joined(separator: " ")) failed")
    }
}
