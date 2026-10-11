import AppKit
@testable import JayJayDiffUI
import XCTest

final class LineSpacingLayoutManagerTests: XCTestCase {
    func testSpacedLinesKeepClicksAndTheTrailingEmptyLineOnTheirOwnRows() {
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        let manager = LineSpacingLayoutManager()
        let container = NSTextContainer(size: NSSize(width: 400, height: CGFloat.greatestFiniteMagnitude))
        let storage = NSTextStorage(string: "alpha\nbeta\ngamma\n", attributes: [.font: font])
        manager.addTextContainer(container)
        storage.addLayoutManager(manager)
        manager.setLineSpacing(1.6, for: font)
        manager.ensureLayout(for: container)

        var height: CGFloat = 0
        for start in [0, 6, 11] {
            let fragment = manager.lineFragmentRect(forGlyphAt: manager.glyphIndexForCharacter(at: start), effectiveRange: nil)
            height = fragment.height
            for y in [fragment.minY + 1, fragment.maxY - 1] {
                let hit = manager.characterIndex(for: NSPoint(x: 0, y: y), in: container, fractionOfDistanceBetweenInsertionPoints: nil)
                XCTAssertEqual(hit, start, "y \(y) belongs to the line starting at \(start)")
            }
        }
        XCTAssertEqual(manager.extraLineFragmentRect.height, height)
    }
}
