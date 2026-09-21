import AppKit
@testable import JayJay
import SwiftUI
import XCTest

@MainActor
final class DescriptionPreviewTests: XCTestCase {
    func testConfiguredFontReachesNativeDescriptionBody() throws {
        let family = try XCTUnwrap(AppSettings.MonoFont(rawValue: "menlo"))
        let window = NSWindow(
            contentRect: CGRect(x: 0, y: 0, width: 400, height: 200),
            styleMask: [.borderless], backing: .buffered, defer: false
        )
        window.isReleasedWhenClosed = false
        defer { window.close() }
        for size in [13.0, 20.0] {
            let body = DescriptionBodyPreview(
                text: "Readable description body",
                collapsedHeight: 80, expandedHeight: 180, expanded: false
            )
            .environment(\.jayjayFontSize, size)
            .environment(\.jayjayFontFamily, family)
            let host = NSHostingView(rootView: body)
            window.contentView = host
            window.layoutIfNeeded()
            RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05))
            let scroll = try XCTUnwrap(descriptionScrollView(in: host))
            XCTAssertEqual(scroll.textView.font?.fontName, family.nsFont(size: size).fontName)
            XCTAssertEqual(scroll.textView.font?.pointSize, CGFloat(size))
        }
    }

    private func descriptionScrollView(in view: NSView) -> DescriptionScrollView? {
        if let scroll = view as? DescriptionScrollView {
            return scroll
        }
        return view.subviews.lazy.compactMap { self.descriptionScrollView(in: $0) }.first
    }

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
