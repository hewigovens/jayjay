@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

final class ShortIdTests: XCTestCase {
    func testPrefixIsBoldAndTheRemainderKeepsTheBaseFont() {
        let font = Font.system(size: 11, design: .monospaced)
        let runs = Array(ShortId(id: "abcdefgh", shortLen: 3).highlighted(scheme: .light, font: font).runs)

        XCTAssertEqual(runs.count, 2)
        XCTAssertEqual(runs.first?.font, font.weight(.bold))
        XCTAssertEqual(runs.last?.font, font)
        XCTAssertEqual(runs.last?.foregroundColor, .secondary)
    }
}
