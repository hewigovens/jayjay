import AppKit
import XCTest

func assertSameColor(
    _ lhs: NSColor?,
    _ rhs: NSColor,
    file: StaticString = #filePath,
    line: UInt = #line
) {
    XCTAssertNotNil(lhs, file: file, line: line)
    guard let lhs else { return }
    assertSameColor(lhs, rhs, file: file, line: line)
}

func assertSameColor(
    _ lhs: NSColor,
    _ rhs: NSColor,
    file: StaticString = #filePath,
    line: UInt = #line
) {
    let left = lhs.usingColorSpace(.deviceRGB)
    let right = rhs.usingColorSpace(.deviceRGB)
    XCTAssertNotNil(left, file: file, line: line)
    XCTAssertNotNil(right, file: file, line: line)
    guard let left, let right else { return }
    XCTAssertEqual(left.redComponent, right.redComponent, accuracy: 0.0001, file: file, line: line)
    XCTAssertEqual(left.greenComponent, right.greenComponent, accuracy: 0.0001, file: file, line: line)
    XCTAssertEqual(left.blueComponent, right.blueComponent, accuracy: 0.0001, file: file, line: line)
    XCTAssertEqual(left.alphaComponent, right.alphaComponent, accuracy: 0.0001, file: file, line: line)
}
