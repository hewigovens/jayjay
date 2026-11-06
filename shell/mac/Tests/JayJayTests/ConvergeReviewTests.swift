@testable import JayJay
import JayJayCore
import XCTest

final class ConvergeReviewTests: XCTestCase {
    func testAuthorsDifferingOnlyByTimestampStayChoosable() {
        let review = ConvergeReview()
        review.accept(plan(authors: [author(at: 1000), author(at: 2000)], author: .unsolved))
        XCTAssertTrue(review.cardItems.contains(.author))
        XCTAssertEqual(review.unresolvedItems, [.author])
        review.focusedItem = .author
        review.useVersion(at: 1)
        XCTAssertTrue(review.canApply)
        XCTAssertEqual(review.choices.author, "b")
    }

    func testOverridingSolvedParentsDropsTheAutomaticContentsPreview() {
        let review = ConvergeReview()
        review.accept(plan(authors: [author(at: 1000), author(at: 1000)], author: .solved(value: author(at: 1000)), parents: [["p1"], ["p2"]]))
        XCTAssertEqual(review.automaticSource(.contents), "a")
        XCTAssertEqual(review.expectedConflicts, ["file.txt"])
        review.use("a", for: .parents)
        XCTAssertEqual(review.automaticSource(.contents), "a")
        review.use("b", for: .parents)
        XCTAssertTrue(review.contentsFollowParents)
        XCTAssertNil(review.automaticSource(.contents))
        XCTAssertNil(review.expectedConflicts)
    }

    private func author(at millis: Int64) -> ConvergeAuthor {
        ConvergeAuthor(name: "Alex", email: "alex@example.test", timestampMillis: millis, timezoneOffsetMinutes: 0)
    }

    private func plan(authors: [ConvergeAuthor], author: ConvergeAuthorAttribute, parents: [[String]] = [["p1"], ["p1"]]) -> ConvergePlan {
        let versions = zip(["a", "b"], zip(authors, parents)).map { id, value in
            ConvergeVersion(commitId: id, description: "same\n", author: value.0, parents: value.1, isImmutable: false, workspaces: [], bookmarks: [])
        }
        return ConvergePlan(
            changeId: "change",
            versions: versions,
            author: author,
            description: .solved(value: "same\n"),
            parents: .solved(value: parents[0]),
            conflictedPaths: ["file.txt"],
            contentsSource: "a",
            effects: ConvergeEffects(descendants: [], bookmarks: [], workspaces: [], blockedReason: nil),
            basis: ["a", "b"]
        )
    }
}
