@testable import JayJay
import XCTest

final class FeedbackEmailTests: XCTestCase {
    @MainActor
    func testOpenUsesAddressAndPrefilledSubject() throws {
        var openedURL: URL?

        XCTAssertTrue(FeedbackEmail.open(using: {
            openedURL = $0
            return true
        }))

        XCTAssertEqual(openedURL?.scheme, "mailto")
        XCTAssertEqual(openedURL?.path, "hi@hewig.dev")
        XCTAssertEqual(
            try URLComponents(url: XCTUnwrap(openedURL), resolvingAgainstBaseURL: false)?.queryItems,
            [URLQueryItem(name: "subject", value: "JayJay Feedback")]
        )
    }

    @MainActor
    func testOpenReportsFailureWhenNoMailClientAcceptsURL() {
        var didReportFailure = false

        XCTAssertFalse(FeedbackEmail.open(using: { _ in false }, onFailure: {
            didReportFailure = true
        }))
        XCTAssertTrue(didReportFailure)
    }
}
