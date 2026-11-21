import JayJayCore
@testable import JayJayReviewUI
import Testing

struct ReaderLineTests {
    private func cell(_ style: RenderedDiff.Style, old: Int? = nil, new: Int? = nil, _ text: String = "x") -> RenderedDiff.Cell {
        RenderedDiff.Cell(oldLine: old.map { .number($0) }, newLine: new, style: style, spans: [.init(text: text)])
    }

    private func diff(_ cells: [RenderedDiff.Cell]) -> RenderedDiff {
        RenderedDiff(rows: cells.map { .init(cells: [$0]) })
    }

    private let file = ReviewFileSummary(path: "a.swift", added: 1, removed: 1)

    @Test func aFoldsPlaceIsTheNewLineItHid() throws {
        let folded = ReaderLine.display(file: file, diff: diff([cell(.context, old: 1, new: 1), cell(.separator, "8 lines")])).lines
        let expanded = ReaderLine.display(file: file, diff: diff((1 ... 10).map { cell(.context, old: $0, new: $0) })).lines
        let fold = try #require(folded.last?.id)
        #expect(fold == .init(side: .fold, number: 2))
        #expect(ReaderLine.closest(fold, in: expanded) == .init(side: .new, number: 2))
    }

    @Test func theFirstPlaceIsJustAboveTheFirstEdit() {
        let lines = ReaderLine.display(file: file, diff: diff([
            cell(.separator, "4 lines"), cell(.context, old: 5, new: 5), cell(.context, old: 6, new: 6),
            cell(.context, old: 7, new: 7), cell(.removed, old: 8), cell(.added, new: 8)
        ])).lines
        #expect(ReaderLine.readingAnchor(saved: nil, in: lines) == .init(side: .new, number: 6))
    }

    @Test func noLinesSayWhy() {
        let huge = ReviewFileSummary(path: "a", added: 10001, removed: 0)
        #expect(ReaderLine.display(file: huge, diff: nil).placeholder?.text == Localized.Web.fileTooLarge)
        let added = ReviewFileSummary(path: "a", kind: .added, added: 0, removed: 0)
        #expect(ReaderLine.display(file: added, diff: diff([])).placeholder?.text == Localized.Web.emptyFile)
        #expect(ReaderLine.display(file: file, diff: nil).placeholder?.text == Localized.Web.noTextChanges)
        let binary = ReviewFileSummary(path: "a", added: 0, removed: 0, notice: "Binary")
        #expect(ReaderLine.display(file: binary, diff: nil).placeholder?.text == "Binary")
    }
}
