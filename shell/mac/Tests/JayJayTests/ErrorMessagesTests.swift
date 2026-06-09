@testable import JayJay
import XCTest

final class ErrorMessagesTests: XCTestCase {
    // Real `jj resolve` failure when the merge tool exits without writing the output file.
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

    // Real `jj resolve --tool :ours` failure on a path with no conflict.
    func testStripsSingleLineWrappers() {
        XCTAssertEqual(
            unwrapCommandError("command failed: Error: No conflicts found at the given path(s)"),
            "No conflicts found at the given path(s)"
        )
    }

    func testLeavesPlainMessageUntouched() {
        XCTAssertEqual(unwrapCommandError("repository is not open"), "repository is not open")
    }
}
