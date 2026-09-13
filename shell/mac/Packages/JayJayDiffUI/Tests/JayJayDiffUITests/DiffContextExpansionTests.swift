import AppKit
import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class DiffContextExpansionTests: XCTestCase {
    func testExpansionLinksRoundTripRequests() {
        for expansion in [ContextExpansion.showMore(lineCount: 10), .showAll] {
            XCTAssertEqual(
                DiffContextExpansionLink.request(
                    from: DiffContextExpansionLink.url(regionId: 42, expansion: expansion)
                ),
                .region(regionId: 42, expansion: expansion)
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
        let coordinator = NativeDiffContextCoordinator()
        var received: ContextExpansionRequest?
        coordinator.onExpandContext = { received = $0 }

        XCTAssertTrue(coordinator.textView(
            NSTextView(),
            clickedOnLink: DiffContextExpansionLink.url(regionId: 9, expansion: .showMore(lineCount: 10)),
            at: 0
        ))
        XCTAssertEqual(received, .region(regionId: 9, expansion: .showMore(lineCount: 10)))
    }

    func testSideBySideCoordinatorDispatchesDecodedRequest() {
        let coordinator = SideBySideCoordinator()
        var received: ContextExpansionRequest?
        coordinator.onExpandContext = { received = $0 }

        XCTAssertTrue(coordinator.textView(
            NSTextView(),
            clickedOnLink: DiffContextExpansionLink.url(regionId: 9, expansion: .showAll),
            at: 0
        ))
        XCTAssertEqual(received, .region(regionId: 9, expansion: .showAll))
    }

    func testRevealFeedbackPolicyHonorsReducedMotionAndLargeReveals() {
        let small = ContextExpansionReveal(
            generation: 1,
            newLines: LineSpan(start: 10, count: 10)
        )
        let large = ContextExpansionReveal(
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
