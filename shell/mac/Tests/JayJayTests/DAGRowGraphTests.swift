import AppKit
@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

@MainActor
final class DAGRowGraphTests: XCTestCase {
    func testMissingAncestryStopsBeforeAReusedLane() throws {
        let entries = [
            entry("A", edges: [GraphEdge(target: "hidden", edgeType: .missing)]),
            entry("B", edges: [GraphEdge(target: "C", edgeType: .direct)]),
            entry("C")
        ]
        let image = try renderGraph(entries, row: 0)

        XCTAssertTrue(alphaValues(image).contains { $0 > 0 })
        XCTAssertTrue(alphaValues(image, fromY: 60).allSatisfy { $0 == 0 })
    }

    func testMixedAncestryCapEndsBeforeParentCurvesFanOut() throws {
        let parent = GraphEdge(target: "P", edgeType: .direct)
        var entries = [entry("H", edges: [parent]), entry("A", edges: [parent]), entry("P")]
        let withoutCap = try renderGraph(entries, row: 1)
        entries[1] = entry("A", edges: [parent, GraphEdge(target: "hidden", edgeType: .missing)])
        let withCap = try renderGraph(entries, row: 1)

        XCTAssertNotEqual(alphaValues(withCap), alphaValues(withoutCap))
        XCTAssertTrue(alphaValues(withCap, fromY: 41) == alphaValues(withoutCap, fromY: 41))
    }

    func testNodeCentersOnTheRefsRowAtAnyFontSize() throws {
        let change = mockChangeInfo(isWorkingCopy: true)
        let layout = DAGLayout(entries: [GraphEntry(change: change, edges: [])])
        let nodeX = dagRowLeadingPadding + layout.xPosition(for: 0, at: 0)

        for fontSize in [12.0, 18.0] {
            let refsRowCenter = try dagRowVerticalPadding + refsRowHeight(change, fontSize: fontSize) / 2
            let node = try nodeSpan(in: dagRow(change).frame(width: 420), atX: nodeX, fontSize: fontSize)

            XCTAssertEqual(
                (node.lowerBound + node.upperBound) / 2,
                refsRowCenter,
                accuracy: 1,
                "node is off the refs-row center at \(fontSize) pt"
            )
        }
    }

    func testChipsWithAnIconKeepTheRefsRowHeight() throws {
        let textOnly = mockChangeInfo(isWorkingCopy: true, hasConflict: true)
        let icons = mockChangeInfo(bookmarks: ["main"], tags: ["v1.0"])
        let conflictedBookmark = mockChangeInfo(bookmarks: ["main"])

        for fontSize in [12.0, 18.0] {
            let expected = try refsRowHeight(textOnly, fontSize: fontSize)

            try XCTAssertEqual(refsRowHeight(icons, fontSize: fontSize), expected, "bookmark and tag chips at \(fontSize) pt")
            try XCTAssertEqual(
                refsRowHeight(conflictedBookmark, conflicted: ["main"], fontSize: fontSize),
                expected,
                "conflicted bookmark chip at \(fontSize) pt"
            )
        }
    }

    private func entry(_ id: String, edges: [GraphEdge] = []) -> GraphEntry {
        GraphEntry(
            change: mockChangeInfo(changeId: id, commitId: id, parents: edges.map(\.target)),
            edges: edges
        )
    }

    private func dagRow(_ change: ChangeInfo, conflicted: Set<String> = []) -> DAGRow {
        let entries = [GraphEntry(change: change, edges: [])]
        return DAGRow(
            viewModel: rowViewModel(entries, index: 0, layout: DAGLayout(entries: entries)),
            conflictedBookmarkNames: conflicted
        )
    }

    private func refsRowHeight(_ change: ChangeInfo, conflicted: Set<String> = [], fontSize: Double) throws -> CGFloat {
        try CGFloat(rendered(dagRow(change, conflicted: conflicted).refsRow.lineLimit(1), fontSize: fontSize).pixelsHigh)
    }

    private func rowViewModel(_ entries: [GraphEntry], index: Int, layout: DAGLayout) -> DAGRowViewModel {
        DAGRowViewModel(
            entry: entries[index],
            layout: layout,
            index: index,
            selectedId: nil,
            compareFromId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )
    }

    private func renderGraph(_ entries: [GraphEntry], row: Int) throws -> NSBitmapImageRep {
        let layout = DAGLayout(entries: entries)
        return try rendered(
            DAGGraphColumn(viewModel: rowViewModel(entries, index: row, layout: layout), nodeCenterY: 12)
                .frame(width: layout.graphWidth, height: 76)
        )
    }

    private func rendered(_ view: some View, fontSize: Double = 12, scale: CGFloat = 1) throws -> NSBitmapImageRep {
        let renderer = ImageRenderer(content: view
            .environment(\.jayjayFontSize, fontSize)
            .environment(\.colorScheme, .light))
        renderer.scale = scale
        return try NSBitmapImageRep(cgImage: XCTUnwrap(renderer.cgImage))
    }

    private func nodeSpan(in view: some View, atX x: CGFloat, fontSize: Double) throws -> ClosedRange<CGFloat> {
        let scale: CGFloat = 2
        let image = try rendered(view, fontSize: fontSize, scale: scale)
        let column = Int(x * scale)
        let inked = (0 ..< image.pixelsHigh).filter { (image.colorAt(x: column, y: $0)?.alphaComponent ?? 0) > 0.6 }
        let first = try XCTUnwrap(inked.first)
        let last = try XCTUnwrap(inked.last)
        return CGFloat(first) / scale ... CGFloat(last + 1) / scale
    }

    private func alphaValues(_ image: NSBitmapImageRep, fromY: Int = 0) -> [CGFloat] {
        (fromY ..< image.pixelsHigh).flatMap { y in
            (0 ..< image.pixelsWide).map { x in
                image.colorAt(x: x, y: y)!.alphaComponent
            }
        }
    }
}
