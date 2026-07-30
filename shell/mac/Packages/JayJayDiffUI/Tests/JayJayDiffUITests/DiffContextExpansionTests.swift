import AppKit
import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class DiffContextExpansionTests: XCTestCase {
    func testExpansionLinksRoundTripRequests() {
        let requests = [
            DiffContextExpansionRequest(
                regionId: 42,
                action: .showMore(lineCount: 10)
            ),
            DiffContextExpansionRequest(regionId: 42, action: .showAll)
        ]

        for request in requests {
            XCTAssertEqual(
                DiffContextExpansionLink.request(
                    from: DiffContextExpansionLink.url(for: request)
                ),
                request
            )
        }
    }

    func testExpansionLinkRejectsForeignAndMalformedURLs() throws {
        XCTAssertNil(try DiffContextExpansionLink.request(from: XCTUnwrap(URL(string: "https://example.com"))))
        XCTAssertNil(try DiffContextExpansionLink.request(
            from: XCTUnwrap(URL(string: "jayjay://diff-context/expand/not-a-number?action=show-all"))
        ))
        XCTAssertNil(try DiffContextExpansionLink.request(
            from: XCTUnwrap(URL(string: "jayjay://diff-context/expand/1?action=show-more&count=0"))
        ))
    }

    func testSeparatorRendersExactlyTwoNativeLinks() {
        let separator = DiffContextExpansionLink.attributedSeparator(
            text: "37 unmodified lines",
            region: ContextRegion(
                id: 7,
                oldStartLine: 11,
                newStartLine: 11,
                lineCount: 37,
                initialLineCount: 37
            ),
            font: .monospacedSystemFont(ofSize: 12, weight: .regular),
            foregroundColor: .secondaryLabelColor
        )
        var linkCount = 0
        separator.enumerateAttribute(
            .link,
            in: NSRange(location: 0, length: separator.length)
        ) { value, _, _ in
            if value != nil {
                linkCount += 1
            }
        }

        XCTAssertEqual(linkCount, 2)
        XCTAssertEqual(
            separator.string,
            "⋯ 37 unmodified lines  Show\u{00A0}10  Show\u{00A0}all\n"
        )
    }

    func testSmallRegionSeparatorOffersOnlyShowAll() {
        let separator = DiffContextExpansionLink.attributedSeparator(
            text: "5 unmodified lines",
            region: ContextRegion(
                id: 7,
                oldStartLine: 11,
                newStartLine: 11,
                lineCount: 5,
                initialLineCount: 5
            ),
            font: .monospacedSystemFont(ofSize: 12, weight: .regular),
            foregroundColor: .secondaryLabelColor
        )
        var linkCount = 0
        separator.enumerateAttribute(
            .link,
            in: NSRange(location: 0, length: separator.length)
        ) { value, _, _ in
            if value != nil {
                linkCount += 1
            }
        }

        XCTAssertEqual(linkCount, 1)
        XCTAssertEqual(
            separator.string,
            "⋯ 5 unmodified lines  Show\u{00A0}all\n"
        )
    }

    func testNativeCoordinatorDispatchesDecodedRequest() {
        let expected = DiffContextExpansionRequest(
            regionId: 9,
            action: .showMore(lineCount: 10)
        )
        let coordinator = NativeDiffContextCoordinator()
        var received: DiffContextExpansionRequest?
        coordinator.onExpandContext = { received = $0 }

        XCTAssertTrue(coordinator.textView(
            NSTextView(),
            clickedOnLink: DiffContextExpansionLink.url(for: expected),
            at: 0
        ))
        XCTAssertEqual(received, expected)
    }

    func testSideBySideCoordinatorDispatchesDecodedRequest() {
        let expected = DiffContextExpansionRequest(regionId: 9, action: .showAll)
        let coordinator = SideBySideCoordinator()
        var received: DiffContextExpansionRequest?
        coordinator.onExpandContext = { received = $0 }

        XCTAssertTrue(coordinator.textView(
            NSTextView(),
            clickedOnLink: DiffContextExpansionLink.url(for: expected),
            at: 0
        ))
        XCTAssertEqual(received, expected)
    }

    func testRevealFeedbackPolicyHonorsReducedMotionAndLargeReveals() {
        let small = DiffContextRevealFeedback(
            generation: 1,
            newLines: LineSpan(start: 10, count: 10)
        )
        let large = DiffContextRevealFeedback(
            generation: 2,
            newLines: LineSpan(start: 10, count: DiffContextRevealFeedbackPolicy.maximumAnimatedLineCount + 1)
        )

        XCTAssertTrue(DiffContextRevealFeedbackPolicy.shouldAnimate(
            feedback: small,
            reduceMotion: false
        ))
        XCTAssertFalse(DiffContextRevealFeedbackPolicy.shouldAnimate(
            feedback: small,
            reduceMotion: true
        ))
        XCTAssertFalse(DiffContextRevealFeedbackPolicy.shouldAnimate(
            feedback: large,
            reduceMotion: false
        ))
    }
}
