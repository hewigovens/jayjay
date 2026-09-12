import AppKit
@testable import JayJay
import XCTest

@MainActor
final class DescriptionPreviewTests: XCTestCase {
    func testTextLayoutUpdatesForWidthAndFontAndPreservesSelectionDuringSizing() {
        let view = DescriptionScrollView()
        view.frame = CGRect(x: 0, y: 0, width: 400, height: 180)
        let text = String(repeating: "A description with enough words to wrap at narrow widths.\n", count: 100)
        view.setDescription(text, font: .monospacedSystemFont(ofSize: 13, weight: .regular))
        let wide = view.contentHeight(for: 600)
        let narrow = view.contentHeight(for: 200)
        XCTAssertGreaterThan(narrow, wide)

        let selection = NSRange(location: 8, length: 12)
        view.textView.setSelectedRange(selection)
        view.setFrameSize(CGSize(width: 200, height: 180))
        view.layoutSubtreeIfNeeded()
        view.contentView.scroll(to: CGPoint(x: 0, y: 80))
        view.setDescription(text, font: .monospacedSystemFont(ofSize: 13, weight: .regular))
        XCTAssertEqual(view.contentHeight(for: 200), narrow)
        XCTAssertEqual(view.textView.selectedRange(), selection)
        XCTAssertEqual(view.contentView.bounds.origin.y, 80)

        view.setDescription(text, font: .monospacedSystemFont(ofSize: 24, weight: .regular))
        XCTAssertGreaterThan(view.contentHeight(for: 200), narrow)
        view.setDescription("Short", font: .monospacedSystemFont(ofSize: 13, weight: .regular))
        XCTAssertLessThan(view.contentHeight(for: 200), 32)
        XCTAssertEqual(view.contentView.bounds.origin.y, 0)
    }
}
