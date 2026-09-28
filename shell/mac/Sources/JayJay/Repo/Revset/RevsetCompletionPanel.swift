import AppKit
import JayJayCore
import SwiftUI

/// Never key: the revset field keeps the keyboard and drives the list.
final class RevsetCompletionPanel: NSPanel {
    private static let maxHeight: CGFloat = 176
    private static let inset: CGFloat = 4

    init() {
        super.init(contentRect: .zero, styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: true)
        isOpaque = false
        backgroundColor = .clear
        hasShadow = true
        isReleasedWhenClosed = false
    }

    override var canBecomeKey: Bool {
        false
    }

    func show(under anchor: NSView, rows: RevsetCompletionRows, fontSize: Double, fontFamily: AppSettings.MonoFont) {
        guard let window = anchor.window else { return }
        let content = ScrollViewReader { proxy in
            ScrollView(.vertical) {
                VStack(alignment: .leading, spacing: 0) {
                    rows
                }
                .padding(.vertical, Self.inset)
            }
            .onChange(of: rows.selected, initial: true) {
                proxy.scrollTo(rows.selected)
            }
        }
        .glassEffect(in: RoundedRectangle(cornerRadius: 12))
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .environment(\.jayjayFontSize, fontSize)
        .environment(\.jayjayFontFamily, fontFamily)
        if let host = contentView as? NSHostingView<AnyView> {
            host.rootView = AnyView(content)
        } else {
            contentView = NSHostingView(rootView: AnyView(content))
        }
        let height = min(
            CGFloat(rows.completions.count) * RevsetCompletionRows.rowHeight + 2 * Self.inset,
            Self.maxHeight
        )
        let bar = window.convertToScreen(anchor.convert(anchor.bounds, to: nil))
        setFrame(NSRect(x: bar.minX, y: bar.minY - height - 4, width: bar.width, height: height), display: true)
        if parent == nil {
            window.addChildWindow(self, ordered: .above)
        }
    }

    func dismiss() {
        parent?.removeChildWindow(self)
        orderOut(nil)
    }
}
