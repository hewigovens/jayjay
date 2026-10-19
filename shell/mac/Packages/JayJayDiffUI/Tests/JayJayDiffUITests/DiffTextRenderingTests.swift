import AppKit
import JayJayCore
@testable import JayJayDiffUI
import SwiftUI
import XCTest

final class DiffTextRenderingTests: XCTestCase {
    func test_changedLineUsesSpanLevelForegroundAndBackground() {
        let theme = DiffPalettes.standard.colors(for: .light)
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        let rendered = NSMutableAttributedString()
        var bgColors: [NSColor] = []

        appendTextLine(
            to: rendered,
            spans: [
                DiffSpan(text: "version = \"0.3.\"", style: .unchanged, token: .plain),
                DiffSpan(text: "6", style: .added, token: .plain)
            ],
            style: .added,
            conflictKind: .none,
            font: font,
            theme: theme,
            bgColors: &bgColors
        )

        let text = rendered.string as NSString
        let unchangedRange = text.range(of: "version")
        let changedRange = text.range(of: "6")

        assertSameColor(colorAttribute(.foregroundColor, at: unchangedRange.location, in: rendered), theme.contextText)
        XCTAssertNil(rendered.attribute(.backgroundColor, at: unchangedRange.location, effectiveRange: nil))
        assertSameColor(colorAttribute(.foregroundColor, at: changedRange.location, in: rendered), theme.addedText)
        XCTAssertNil(rendered.attribute(.backgroundColor, at: changedRange.location, effectiveRange: nil))
        assertSameColor(colorAttribute(.diffWordHighlightColor, at: changedRange.location, in: rendered), theme.addedWordBg)
        assertSameColor(bgColors.first, theme.addedBg)
    }

    func test_changedSpanForegroundWinsOverSyntaxTokenColor() {
        let theme = DiffPalettes.standard.colors(for: .light)
        let changedString = DiffSpan(text: "6", style: .added, token: .stringLit)
        let unchangedString = DiffSpan(text: "\"0.3.\"", style: .unchanged, token: .stringLit)

        assertSameColor(theme.spanText(changedString, lineStyle: .added, conflictKind: .none), theme.addedText)
        assertSameColor(theme.spanText(unchangedString, lineStyle: .added, conflictKind: .none), theme.string)
    }

    func test_binarySizeTooltipReachesBothNativeRenderers() {
        let theme = DiffPalettes.standard.colors(for: .light)
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        let spans = [DiffSpan(text: "<binary file (141080 bytes)>", style: .added, token: .plain)]
        let line = DiffLine(
            oldLineNo: nil,
            newLineNo: 1,
            style: .added,
            spans: spans,
            conflictKind: .none,
            noEofNewline: false,
            contextRegion: nil
        )
        let unified = NSMutableAttributedString()
        let sideBySide = NSMutableAttributedString()
        var colors: [NSColor] = []
        var locations: [DiffViewportLineLocation] = []
        appendNativeDiffLine(
            line,
            to: unified,
            context: NativeDiffLineRenderContext(font: font, theme: theme, enablesContextExpansion: false),
            bgColors: &colors,
            viewportLineLocations: &locations
        )
        appendTextLine(
            to: sideBySide,
            spans: spans,
            style: .added,
            conflictKind: .none,
            font: font,
            theme: theme,
            bgColors: &colors
        )
        for rendered in [unified, sideBySide] {
            XCTAssertTrue(rendered.string.contains("KB"))
            XCTAssertFalse(rendered.string.contains("bytes"))
            let expected = "\(141_080.formatted(.number.locale(.current))) bytes"
            XCTAssertEqual(rendered.attribute(.toolTip, at: 0, effectiveRange: nil) as? String, expected)
        }
    }

    private func colorAttribute(
        _ key: NSAttributedString.Key,
        at index: Int,
        in rendered: NSAttributedString
    ) -> NSColor? {
        rendered.attribute(key, at: index, effectiveRange: nil) as? NSColor
    }
}
