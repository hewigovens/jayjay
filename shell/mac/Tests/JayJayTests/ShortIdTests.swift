@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

final class ShortIdTests: XCTestCase {
    func testCompactDisplayPreservesTheUniquePrefix() {
        let font = Font.system(size: 11, design: .monospaced)
        for (prefixLength, expected) in [(3, "abcdefgh"), (10, "abcdefghij")] {
            let rendered = ShortId(id: "abcdefghijkl", shortLen: UInt32(prefixLength))
                .highlighted(scheme: .dark, font: font)
            XCTAssertEqual(String(rendered.characters), expected)
        }
    }

    func testPrefixIsBoldAndTheRemainderKeepsTheBaseFont() {
        let font = Font.system(size: 11, design: .monospaced)
        let runs = Array(ShortId(id: "abcdefgh", shortLen: 3).highlighted(scheme: .light, font: font).runs)

        XCTAssertEqual(runs.count, 2)
        XCTAssertEqual(runs.first?.font, font.weight(.bold))
        XCTAssertEqual(runs.last?.font, font)
        XCTAssertEqual(runs.last?.foregroundColor, .secondary)
    }
}
