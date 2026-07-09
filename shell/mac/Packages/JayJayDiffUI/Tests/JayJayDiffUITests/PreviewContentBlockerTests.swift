@testable import JayJayDiffUI
import WebKit
import XCTest

@MainActor
final class PreviewContentBlockerTests: XCTestCase {
    func testRuleListTargetsHTTPAndHTTPSOnly() throws {
        let json = try JSONSerialization.jsonObject(
            with: Data(PreviewContentBlocker.ruleListJSON.utf8)
        ) as? [[String: Any]]

        let rule = try XCTUnwrap(json?.first)
        let trigger = try XCTUnwrap(rule["trigger"] as? [String: String])
        let action = try XCTUnwrap(rule["action"] as? [String: String])

        XCTAssertEqual(trigger["url-filter"], "^https?://")
        XCTAssertEqual(action["type"], "block")
    }

    func testApplyCompilesAndAddsRuleListToConfiguration() {
        let configuration = WKWebViewConfiguration()
        let expectation = expectation(description: "content rule list applied")
        var compiled: WKContentRuleList?

        PreviewContentBlocker.apply(to: configuration) { ruleList in
            compiled = ruleList
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 5)
        XCTAssertNotNil(compiled, "rule list should compile and be added to the configuration")
    }

    func testSanitizedHTMLAlwaysProceedsRegardlessOfBlockerState() {
        for state: PreviewContentBlocker.State in [.pending, .ready, .unavailable] {
            XCTAssertEqual(
                PreviewContentBlocker.loadDecision(for: .sanitizedHTML, blockerState: state),
                .proceed
            )
        }
    }

    func testRawHTMLWaitsWhileBlockerIsPending() {
        XCTAssertEqual(
            PreviewContentBlocker.loadDecision(for: .rawHTML, blockerState: .pending),
            .wait
        )
    }

    func testRawHTMLProceedsOnceBlockerIsReady() {
        XCTAssertEqual(
            PreviewContentBlocker.loadDecision(for: .rawHTML, blockerState: .ready),
            .proceed
        )
    }

    func testRawHTMLFailsClosedWhenBlockerIsUnavailable() {
        // Raw HTML has no other guard against remote subresources, so a failed compile must not render it.
        XCTAssertEqual(
            PreviewContentBlocker.loadDecision(for: .rawHTML, blockerState: .unavailable),
            .failClosed
        )
    }
}
