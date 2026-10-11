import AppKit
@testable import JayJay
import SwiftUI
import XCTest

@MainActor
final class CodeTextViewTypographyTests: XCTestCase {
    private final class Model: ObservableObject {
        @Published var typography = Typography()
    }

    private struct Host: View {
        @ObservedObject var model: Model
        @State var text: String

        var body: some View {
            CodeTextView(path: "file.txt", text: $text, isEditable: true, wrapsLines: false, mergePane: .right)
                .frame(width: 400, height: 200)
                .environment(\.jayjayTypography, model.typography)
        }
    }

    private func descendants(_ view: NSView) -> [NSView] {
        view.subviews.flatMap { [$0] + descendants($0) }
    }

    private func settle(_ host: NSView) {
        host.layoutSubtreeIfNeeded()
        for _ in 0 ..< 5 {
            RunLoop.main.run(until: Date().addingTimeInterval(0.02))
        }
    }

    func testTypographyChangeKeepsTextSelectionAndTopLine() throws {
        let model = Model()
        let host = NSHostingView(rootView: Host(model: model, text: (1 ... 200).map { "line \($0)" }.joined(separator: "\n")))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 400, height: 200), styleMask: [.borderless], backing: .buffered, defer: false)
        window.contentView = host
        settle(host)
        let editor = try XCTUnwrap(descendants(host).compactMap { $0 as? NSTextView }.first)
        let scroll = try XCTUnwrap(editor.enclosingScrollView)
        editor.insertText("edited ", replacementRange: NSRange(location: 0, length: 0))
        let selection = NSRange(location: 900, length: 4)
        editor.setSelectedRange(selection)
        let layout = try XCTUnwrap(editor.layoutManager)
        let container = try XCTUnwrap(editor.textContainer)
        let topLine = { () -> Int in
            let top = NSPoint(x: 0, y: scroll.contentView.bounds.minY - editor.textContainerOrigin.y + 1)
            let index = layout.characterIndex(for: top, in: container, fractionOfDistanceBetweenInsertionPoints: nil)
            return (editor.string as NSString).substring(to: index).filter { $0 == "\n" }.count
        }
        scroll.contentView.scroll(to: NSPoint(x: 0, y: 1200))
        scroll.reflectScrolledClipView(scroll.contentView)
        let text = editor.string
        let line = topLine()
        model.typography = Typography(codeSize: 18, codeLineSpacing: 1.5)
        settle(host)
        XCTAssertEqual(editor.font?.pointSize, 18)
        XCTAssertEqual(editor.string, text)
        XCTAssertEqual(editor.selectedRange(), selection)
        XCTAssertEqual(topLine(), line)
    }
}
