import AppKit
@testable import JayJay
import XCTest

final class TypographyTests: XCTestCase {
    func testInterfaceAndCodeTextUseTheirOwnFamilyAndSize() throws {
        let typography = try Typography(
            interfaceFamily: XCTUnwrap(AppSettings.FontFamily(rawValue: "Courier New")),
            interfaceSize: 18,
            codeFamily: XCTUnwrap(AppSettings.FontFamily(rawValue: "menlo")),
            codeSize: 15
        )
        let interface = typography.nsFont(12)
        let code = typography.nsFont(12, monospaced: true)
        XCTAssertEqual([interface.familyName, code.familyName], ["Courier New", "Menlo"])
        XCTAssertEqual([interface.pointSize, code.pointSize], [18, 15])
        XCTAssertEqual(typography.editorFont.pointSize, 15)
    }

    func testLigaturesAreDetectedAndCanBeTurnedOff() throws {
        let family = try XCTUnwrap(AppSettings.FontFamily(rawValue: "Helvetica Neue"))
        func glyphCount(ligatures: Bool) throws -> Int {
            let font = family.nsFont(size: 12, monospaced: false, ligatures: ligatures)
            let line = CTLineCreateWithAttributedString(NSAttributedString(string: "fi", attributes: [.font: font]))
            return CTLineGetGlyphCount(line)
        }
        XCTAssertEqual(try glyphCount(ligatures: true), 1)
        XCTAssertEqual(try glyphCount(ligatures: false), 2)
        XCTAssertTrue(family.hasLigatures)
        XCTAssertFalse(try XCTUnwrap(AppSettings.FontFamily(rawValue: "menlo")).hasLigatures)
        XCTAssertFalse(AppSettings.FontFamily.system.hasLigatures)
    }
}
