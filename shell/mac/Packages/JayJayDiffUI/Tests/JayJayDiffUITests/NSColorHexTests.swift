import AppKit
@testable import JayJayDiffUI
import XCTest

final class NSColorHexTests: XCTestCase {
    func test_hexInitializerUsesSrgbComponents() {
        guard let color = NSColor(hex: 0x336699, alpha: 0.4).usingColorSpace(.sRGB) else {
            XCTFail("Expected sRGB color")
            return
        }

        XCTAssertEqual(color.redComponent, CGFloat(0x33) / 255, accuracy: 0.001)
        XCTAssertEqual(color.greenComponent, CGFloat(0x66) / 255, accuracy: 0.001)
        XCTAssertEqual(color.blueComponent, CGFloat(0x99) / 255, accuracy: 0.001)
        XCTAssertEqual(color.alphaComponent, 0.4, accuracy: 0.001)
    }
}
