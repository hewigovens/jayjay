@testable import JayJay
import XCTest

final class ErrorMessagesTests: XCTestCase {
    /// Real `jj resolve` failure when the merge tool exits without writing the output file.
    func testReducesResolveFailureToPlainSentence() {
        let raw = """
        command failed: Resolving conflicts in: f.txt
        Error: Failed to resolve conflicts
        Caused by: The output file is either unchanged or empty after the editor quit (run with --debug to see the exact invocation).
        """
        XCTAssertEqual(
            unwrapCommandError(raw),
            "Failed to resolve conflicts: The output file is either unchanged or empty after the editor quit"
        )
    }

    /// Real `jj resolve --tool :ours` failure on a path with no conflict.
    func testStripsSingleLineWrappers() {
        XCTAssertEqual(
            unwrapCommandError("command failed: Error: No conflicts found at the given path(s)"),
            "No conflicts found at the given path(s)"
        )
    }

    func testPushFailureKeepsTransportDetails() {
        let raw = """
        git push failed: Changes to push to origin:
          bookmark: main [advance 0bb004e -> 7517127]
        git: git@github.com: Permission denied (publickey).
        git:
        Error: Failed to push some bookmarks
        """
        XCTAssertEqual(
            unwrapCommandError(raw),
            """
            Failed to push some bookmarks
            git: git@github.com: Permission denied (publickey).
            """
        )
    }

    func testPushFailureKeepsExternalGitDetails() {
        let raw = """
        git push failed: Changes to push to origin:
          bookmark: main [add to 9d084bdd0c7e]
        git: ssh: Could not resolve hostname invalid.invalid: nodename nor servname provided, or not known
        git:
        Error: Git process failed: External git program failed:
        fatal: Could not read from remote repository.
        """
        XCTAssertEqual(
            unwrapCommandError(raw),
            """
            Git process failed: External git program failed:
            git: ssh: Could not resolve hostname invalid.invalid: nodename nor servname provided, or not known
            fatal: Could not read from remote repository.
            """
        )
    }

    func testLeavesPlainMessageUntouched() {
        XCTAssertEqual(unwrapCommandError("repository is not open"), "repository is not open")
    }
}
