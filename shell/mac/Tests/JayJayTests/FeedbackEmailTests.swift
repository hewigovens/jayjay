@testable import JayJay
import XCTest

final class FeedbackEmailTests: XCTestCase {
    @MainActor
    func testOpenUsesAddressAndPrefilledSubject() throws {
        var openedURL: URL?

        XCTAssertTrue(FeedbackEmail.open(attaching: nil, using: {
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

        XCTAssertFalse(FeedbackEmail.open(attaching: nil, using: { _ in false }, onFailure: {
            didReportFailure = true
        }))
        XCTAssertTrue(didReportFailure)
    }

    func testComposeURLAsksForTheDebugLog() throws {
        let url = FeedbackEmail.composeURL(askingFor: URL(fileURLWithPath: "/logs/jayjay.log"))
        let body = try URLComponents(url: url, resolvingAgainstBaseURL: false)?
            .queryItems?.first { $0.name == "body" }?.value
        XCTAssertEqual(body, "Please attach the debug log: /logs/jayjay.log\n\n")
    }
}
