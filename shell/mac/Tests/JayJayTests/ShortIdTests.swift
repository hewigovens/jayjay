@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

final class ShortIdTests: XCTestCase {
    func testCompactIdsKeepTheUniquePrefix() {
        let font = Font.system(size: 11, design: .monospaced)
        for (id, prefix, expected) in [("abcdefghijklmnop", 3, "abcdefgh"), ("abcdefghijklmnop", 10, "abcdefghij"), ("abcd", 8, "abcd")] {
            let shortId = ShortId(id: id, shortLen: UInt32(prefix))
            XCTAssertEqual(shortId.compact, expected)
            XCTAssertEqual(String(shortId.highlighted(scheme: .light, font: font).characters), expected)
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
