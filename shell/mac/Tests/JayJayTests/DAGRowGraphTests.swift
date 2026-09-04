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

    private func entry(_ id: String, edges: [GraphEdge] = []) -> GraphEntry {
        GraphEntry(
            change: mockChangeInfo(changeId: id, commitId: id, parents: edges.map(\.target)),
            edges: edges
        )
    }

    private func renderGraph(_ entries: [GraphEntry], row: Int) throws -> NSBitmapImageRep {
        let layout = DAGLayout(entries: entries)
        let viewModel = DAGRowViewModel(
            entry: entries[row],
            layout: layout,
            index: row,
            selectedId: nil,
            compareFromId: nil,
            contextTargetId: nil,
            rebaseDrag: nil,
            rebasePreviewText: nil,
            bookmarkDrag: nil,
            bookmarkPreviewText: nil,
            colorScheme: .light
        )
        let renderer = ImageRenderer(content: DAGRow(viewModel: viewModel).graphColumn
            .frame(width: layout.graphWidth, height: 76)
            .environment(\.colorScheme, .light))
        return try NSBitmapImageRep(cgImage: XCTUnwrap(renderer.cgImage))
    }

    private func alphaValues(_ image: NSBitmapImageRep, fromY: Int = 0) -> [CGFloat] {
        (fromY ..< image.pixelsHigh).flatMap { y in
            (0 ..< image.pixelsWide).map { x in
                image.colorAt(x: x, y: y)!.alphaComponent
            }
        }
    }
}
