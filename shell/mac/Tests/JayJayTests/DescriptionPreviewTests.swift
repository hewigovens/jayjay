import AppKit
@testable import JayJay
import XCTest

@MainActor
final class DescriptionPreviewTests: XCTestCase {
    func testTextLayoutUpdatesForWidthAndFontAndPreservesSelectionDuringSizing() {
        let view = DescriptionScrollView()
        let titleFont = NSFont.systemFont(ofSize: 14, weight: .semibold)
        view.frame = CGRect(x: 0, y: 0, width: 400, height: 180)
        let text = String(repeating: "A description with enough words to wrap at narrow widths.\n", count: 100)
        view.setDescription(text, font: .monospacedSystemFont(ofSize: 13, weight: .regular), titleFont: titleFont)
        let styled = view.textView.attributedString()
        XCTAssertEqual(styled.attribute(.font, at: 0, effectiveRange: nil) as? NSFont, titleFont)
        let secondLine = (text as NSString).range(of: "\n").location + 1
        let bodyFont = styled.attribute(.font, at: secondLine, effectiveRange: nil) as? NSFont
        XCTAssertLessThan(bodyFont?.pointSize ?? 0, titleFont.pointSize)
        let wide = view.contentHeight(for: 600)
        let narrow = view.contentHeight(for: 200)
        XCTAssertGreaterThan(narrow, wide)

        let selection = NSRange(location: 8, length: 12)
        view.textView.setSelectedRange(selection)
        view.contentView.scroll(to: CGPoint(x: 0, y: 80))
        view.setDescription(text, font: .monospacedSystemFont(ofSize: 13, weight: .regular), titleFont: titleFont)
        XCTAssertEqual(view.contentHeight(for: 200), narrow)
        XCTAssertEqual(view.textView.selectedRange(), selection)
        XCTAssertEqual(view.contentView.bounds.origin.y, 80)

        view.setDescription(text, font: .monospacedSystemFont(ofSize: 24, weight: .regular), titleFont: titleFont)
        XCTAssertGreaterThan(view.contentHeight(for: 200), narrow)
        view.setDescription("Short", font: .monospacedSystemFont(ofSize: 13, weight: .regular), titleFont: titleFont)
        XCTAssertLessThan(view.contentHeight(for: 200), 32)
        XCTAssertEqual(view.contentView.bounds.origin.y, 0)
    }

    func testEditButtonFollowsTitleAndStaysInsideWrappedPreview() throws {
        let view = DescriptionScrollView()
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        let titleFont = NSFont.systemFont(ofSize: 14, weight: .semibold)
        view.onEdit = {}
        for title in ["Short title", String(repeating: "Long title ", count: 12)] {
            view.setDescription(title + "\n\nBody", font: font, titleFont: titleFont)
            _ = view.contentHeight(for: 400)
            let layout = try XCTUnwrap(view.textView.layoutManager)
            let line = layout.lineFragmentUsedRect(forGlyphAt: 0, effectiveRange: nil)
            XCTAssertEqual(view.editButton.frame.minX, line.maxX + 8, accuracy: 1)
            XCTAssertLessThanOrEqual(view.editButton.frame.maxX, 400 - NSScroller.scrollerWidth(for: .regular, scrollerStyle: .overlay))
            XCTAssertLessThan(view.editButton.frame.maxY, 32)
            XCTAssertEqual(view.textView.frame.width, 400)
            XCTAssertEqual(view.textView.string, title + "\n\nBody")
        }
        view.onEdit = nil
        view.setDescription("Short title", font: font, titleFont: titleFont)
        XCTAssertTrue(view.editButton.isHidden)
    }
}
