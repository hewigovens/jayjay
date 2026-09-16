@testable import JayJay
import XCTest

final class AppleIntelligenceTests: XCTestCase {
    func testGeneratedCommitMessageBecomesSummaryPlusBulletBody() {
        let generated = GeneratedCommitMessage(
            summary: "Fix: resolve crash on empty diff view",
            bullets: ["  Handle a nil layout manager  ", "Add a bounds check"]
        )

        XCTAssertEqual(generated.message, """
        Fix: resolve crash on empty diff view

        - Handle a nil layout manager
        - Add a bounds check
        """)
    }

    func testGeneratedCommitMessageWithoutBulletsIsJustTheSummary() {
        let generated = GeneratedCommitMessage(summary: "Chore: bump jj to 0.45", bullets: [])

        XCTAssertEqual(generated.message, "Chore: bump jj to 0.45")
    }
}
