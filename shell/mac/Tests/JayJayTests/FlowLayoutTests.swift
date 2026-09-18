@testable import JayJay
import XCTest

final class FlowLayoutTests: XCTestCase {
    private let layout = FlowLayout(spacing: 6, lineSpacing: 4)

    func testSubviewsWrapOnceTheLineRunsOut() {
        let plan = layout.plan(sizes: Array(repeating: CGSize(width: 40, height: 20), count: 3), maxWidth: 100)

        XCTAssertEqual(plan.frames.map(\.origin), [CGPoint(x: 0, y: 0), CGPoint(x: 46, y: 0), CGPoint(x: 0, y: 24)])
        XCTAssertEqual(plan.size, CGSize(width: 86, height: 44))
    }

    func testOneLineReportsItsPackedSize() {
        let plan = layout.plan(sizes: [CGSize(width: 40, height: 20), CGSize(width: 30, height: 24)], maxWidth: 200)

        XCTAssertEqual(plan.size, CGSize(width: 76, height: 24))
    }

    func testASubviewWiderThanTheLineStartsItsOwnLine() {
        let plan = layout.plan(sizes: [CGSize(width: 40, height: 20), CGSize(width: 300, height: 20)], maxWidth: 100)

        XCTAssertEqual(plan.frames.map(\.origin), [CGPoint(x: 0, y: 0), CGPoint(x: 0, y: 24)])
    }
}
