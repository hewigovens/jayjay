@testable import JayJayDiffUI
import XCTest

final class MarkdownDiffViewTests: XCTestCase {
    func testSmallMarkdownUsesRichRenderer() {
        XCTAssertEqual(
            MarkdownDiffView.renderPlan(for: "# Title\n\nBody"),
            .rich("# Title\n\nBody")
        )
    }

    func testLargeMarkdownAvoidsRichRenderer() {
        let markdown = String(repeating: "notebook cell\n", count: 30_000)

        let plan = MarkdownDiffView.renderPlan(for: markdown)

        guard case let .plainPreview(preview, truncated) = plan else {
            return XCTFail("expected large markdown to avoid Textual rich renderer")
        }
        XCTAssertTrue(truncated)
        XCTAssertEqual(preview.count, MarkdownDiffView.largePlainPreviewCharacterLimit)
    }

    func testEmptyMarkdownShowsEmptyState() {
        XCTAssertEqual(MarkdownDiffView.renderPlan(for: nil), .empty)
        XCTAssertEqual(MarkdownDiffView.renderPlan(for: ""), .empty)
    }
}
