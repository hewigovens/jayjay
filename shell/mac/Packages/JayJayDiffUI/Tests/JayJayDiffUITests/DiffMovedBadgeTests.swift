import AppKit
import JayJayCore
@testable import JayJayDiffUI
import SwiftUI
import XCTest

@MainActor
final class DiffMovedBadgeTests: XCTestCase {
    func testReusedButtonUsesLatestCallbackAndUpdatesChangedLabelAndHeight() {
        let button = DiffMovedBadgeButton()
        let smallFont = DiffMovedBadgeButton.fittedFont(.systemFont(ofSize: 11), maxHeight: 10)
        button.configure(title: "→ L5", font: smallFont) { XCTFail("stale callback") }
        let small = button.fittingSize
        var pressed = false
        button.configure(title: "→ L5", font: smallFont) { pressed = true }
        button.performClick(nil)
        XCTAssertTrue(pressed)

        button.configure(title: "→ L123456", font: smallFont) {}
        XCTAssertTrue(button.attributedTitle.string.contains("L123456"))
        XCTAssertGreaterThan(button.fittingSize.width, small.width)
        XCTAssertLessThanOrEqual(button.fittingSize.height, 10)

        button.configure(title: "→ L123456", font: .systemFont(ofSize: 11)) {}
        XCTAssertGreaterThan(button.fittingSize.height, small.height)
        XCTAssertLessThanOrEqual(button.fittingSize.height, 14)
    }

    func testBadgeLineLookupUsesSourceLFBoundaries() {
        let first = "before\u{2028}same\rsource😀"
        let second = "moved code"
        let text = "\(first)\n\(second)\n" as NSString
        let starts = text.lineStarts()
        XCTAssertEqual(starts.count, 2)
        XCTAssertEqual(starts[0], 0)
        XCTAssertEqual(starts[1], (first as NSString).length + 1)
        XCTAssertEqual(text.substring(from: starts[1]), "\(second)\n")
    }

    func testNativeFirstFragmentReservesSpaceAndContinuationsRestoreWidth() throws {
        let text = String(repeating: "a", count: 45)
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        let advance = ("a" as NSString).size(withAttributes: [.font: font]).width
        let container = DiffTextContainer(containerSize: NSSize(width: 30 * advance, height: 1000))
        container.lineFragmentPadding = 0
        container.widthTracksTextView = false
        let storage = NSTextStorage(string: text, attributes: [.font: font])
        let manager = NSLayoutManager()
        manager.addTextContainer(container)
        storage.addLayoutManager(manager)
        container.reserveMovedBadgeSpace(
            for: [DiffMovedBadge(line: 0, title: "→ L9", target: 1)],
            in: text as NSString,
            font: .systemFont(ofSize: 11)
        )
        manager.ensureLayout(for: container)
        var widths: [CGFloat] = []
        manager.enumerateLineFragments(forGlyphRange: NSRange(location: 0, length: manager.numberOfGlyphs)) { rect, _, _, _, _ in
            widths.append(rect.width)
        }
        XCTAssertGreaterThan(widths.count, 1)
        XCTAssertLessThan(try XCTUnwrap(widths.first), 30 * advance)
        for width in widths.dropFirst() {
            XCTAssertEqual(width, 30 * advance, accuracy: 0.01)
        }
        container.reserveMovedBadgeSpace(for: [], in: text as NSString, font: .systemFont(ofSize: 11))
        manager.ensureLayout(for: container)
        XCTAssertEqual(manager.lineFragmentRect(forGlyphAt: 0, effectiveRange: nil).width, 30 * advance, accuracy: 0.01)
    }

    func testBothLayoutsUpdateBadgesWhenOnlyInterfaceTypographyChanges() throws {
        _ = NSApplication.shared
        func line(_ style: DiffSpanStyle, _ old: UInt32?, _ new: UInt32?, _ moved: MovedRow? = nil) -> DiffLine {
            DiffLine(
                oldLineNo: old,
                newLineNo: new,
                style: style,
                spans: [DiffSpan(text: String(repeating: "x", count: 70), style: style, token: .plain)],
                conflictKind: .none,
                noEofNewline: false,
                contextRegion: nil,
                moved: moved
            )
        }
        let diff = FileDiff(path: "moved.rs", language: "rust", lines: [
            line(.removed, 1, nil, MovedRow(counterpartLine: 9, startsBlock: true)),
            line(.context, 2, 5), line(.context, 3, 6), line(.context, 4, 7),
            line(.added, nil, 9, MovedRow(counterpartLine: 1, startsBlock: true))
        ], whitespaceOnlyHidden: false)
        let fonts = try [XCTUnwrap(NSFont(name: "Courier New", size: 11)), XCTUnwrap(NSFont(name: "Georgia", size: 16.5))]
        for sideBySide in [false, true] {
            func view(_ interfaceFont: NSFont) -> some View {
                Group {
                    if sideBySide {
                        SideBySideDiffView(diff: diff)
                    } else {
                        NativeDiffView(diff: diff)
                    }
                }
                .environment(\.diffTypography, DiffTypography(
                    font: .monospacedSystemFont(ofSize: 24, weight: .regular),
                    interfaceFont: interfaceFont
                ))
            }
            let hosting = NSHostingView(rootView: view(fonts[0]))
            let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 700, height: 350), styleMask: [.titled], backing: .buffered, defer: false)
            window.contentView = hosting
            for font in fonts {
                hosting.rootView = view(font)
                hosting.layoutSubtreeIfNeeded()
                RunLoop.main.run(until: Date().addingTimeInterval(0.05))
                hosting.layoutSubtreeIfNeeded()
                func buttons(in view: NSView) -> [DiffMovedBadgeButton] {
                    (view as? DiffMovedBadgeButton).map { [$0] } ?? view.subviews.flatMap { buttons(in: $0) }
                }
                let badges = buttons(in: hosting).filter { !$0.isHidden }
                XCTAssertEqual(badges.count, 2)
                for badge in badges {
                    XCTAssertEqual(badge.attributedTitle.attribute(.font, at: 0, effectiveRange: nil) as? NSFont, font)
                }
            }
            window.contentView = nil
        }
    }
}
