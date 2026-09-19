import AppKit
@testable import JayJay
import SwiftUI
import XCTest

@MainActor
final class CodeLineNumberRulerTests: XCTestCase {
    func testNumbersFollowLogicalLinesAcrossWrappingScrollingAndEdits() throws {
        let scroll = NSScrollView(frame: NSRect(x: 0, y: 0, width: 220, height: 100))
        let view = NSTextView(frame: scroll.bounds)
        view.font = CodeTextView.editorFont
        view.isVerticallyResizable = true
        view.textContainer?.widthTracksTextView = true
        view.textContainer?.containerSize = NSSize(width: 180, height: CGFloat.greatestFiniteMagnitude)
        view.string = String(repeating: "日本語 🐦 ", count: 30) + "\r\nsecond\n\nlast\n"
        scroll.documentView = view
        let ruler = CodeLineNumberRuler(scrollView: scroll, textView: view)
        scroll.verticalRulerView = ruler
        scroll.hasVerticalRuler = true
        scroll.rulersVisible = true
        view.sizeToFit()
        let labels = ruler.lineLabels(in: view.bounds)
        XCTAssertEqual(labels.map(\.number), [1, 2, 3, 4, 5])
        let second = try XCTUnwrap(labels.first { $0.number == 2 })
        XCTAssertGreaterThan(second.origin.y - labels[0].origin.y, 30)
        let visible = NSRect(x: 0, y: second.origin.y, width: view.bounds.width, height: 100)
        scroll.contentView.scroll(to: visible.origin)
        XCTAssertEqual(ruler.lineLabels(in: visible).map(\.number), [2, 3, 4, 5])
        XCTAssertEqual(ruler.convert(second.origin, from: view).y, 0, accuracy: 0.5)
        view.string = "inserted\n" + view.string
        ruler.updateText()
        view.sizeToFit()
        XCTAssertEqual(ruler.lineLabels(in: view.bounds).map(\.number), [1, 2, 3, 4, 5, 6])
        view.string = ""
        ruler.updateText()
        view.sizeToFit()
        XCTAssertEqual(ruler.lineLabels(in: NSRect(x: 0, y: 0, width: 200, height: 100)).map(\.number), [1])
    }

    func testHostedPaneKeepsRulerBelowHeader() throws {
        let text = (1 ... 100).map { "line \($0)" }.joined(separator: "\n")
        let host = NSHostingView(rootView: VStack(spacing: 0) {
            Text("Right").frame(height: 30)
            CodeTextView(path: "file.txt", text: .constant(text), isEditable: false, wrapsLines: true, mergePane: .right)
        }.frame(width: 400, height: 200))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 400, height: 200), styleMask: [.borderless], backing: .buffered, defer: false)
        window.contentView = host
        host.layoutSubtreeIfNeeded()
        window.displayIfNeeded()
        func descendants(_ view: NSView) -> [NSView] {
            view.subviews.flatMap { [$0] + descendants($0) }
        }
        let ruler = try XCTUnwrap(descendants(host).compactMap { $0 as? CodeLineNumberRuler }.first)
        let scroll = try XCTUnwrap(ruler.scrollView)
        scroll.contentView.scroll(to: NSPoint(x: 0, y: 127))
        host.layoutSubtreeIfNeeded()
        let visible = ruler.convert(ruler.visibleRect, to: host)
        XCTAssertGreaterThanOrEqual(visible.minY, 30)
        XCTAssertLessThanOrEqual(visible.maxY, host.bounds.maxY)
        XCTAssertLessThanOrEqual(visible.width, ruler.ruleThickness)
    }
}
