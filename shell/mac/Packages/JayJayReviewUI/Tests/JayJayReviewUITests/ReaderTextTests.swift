import Foundation
import JayJayCore
@testable import JayJayReviewUI
import Testing

@MainActor struct ReaderTextTests {
    private func cell(_ json: String) throws -> RenderedDiff.Cell {
        try JSONDecoder().decode(RenderedDiff.Cell.self, from: Data(json.utf8))
    }

    private func highlighted(_ text: AttributedString) -> [String] {
        text.runs.filter { $0.backgroundColor != nil }.map { String(text[$0.range].characters) }
    }

    @Test func aPairedReplacementHighlightsOnlyTheChangedDigits() throws {
        for (style, digit) in [("removed", "4"), ("added", "8")] {
            let c = try cell("""
            {"style":"\(style)","spans":[{"text":"    let limit = ","style":"unchanged","token":"Plain"},
            {"text":"\(digit)","style":"\(style)","token":"Number"}],"noEofNewline":false}
            """)
            let text = UnifiedDiffReader.code(c, theme: ColorTheme.standard(dark: false).colors)
            #expect(String(text.characters) == "    let limit = " + digit)
            #expect(highlighted(text) == [digit])
        }
    }

    @Test func contextAndUnknownMetadataStayVerbatimWithoutInventedEdits() throws {
        for style in ["context", "added"] {
            let c = try cell("""
            {"style":"\(style)","spans":[{"text":"    值"},{"text":" 👩🏻‍💻","style":"futureStyle"},
            {"text":" ","style":"unchanged","token":"Keyword"}],"noEofNewline":true}
            """)
            let text = UnifiedDiffReader.code(c, theme: ColorTheme.standard(dark: false).colors)
            #expect(String(text.characters) == "    值 👩🏻‍💻  ↵ " + Localized.Web.eof)
            #expect(highlighted(text).isEmpty)
        }
        let context = try cell(#"{"style":"context","spans":[{"text":"neutral","style":"added"}],"noEofNewline":false}"#)
        #expect(highlighted(UnifiedDiffReader.code(context, theme: ColorTheme.standard(dark: true).colors)).isEmpty)
    }
}
