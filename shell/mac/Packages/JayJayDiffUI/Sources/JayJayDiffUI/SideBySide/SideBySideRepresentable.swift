import AppKit
import JayJayCore
import SwiftUI

public struct SideBySideRepresentable: NSViewRepresentable {
    public typealias Coordinator = SideBySideCoordinator

    public let diff: FileDiff

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.diffFontSize) private var fontSize
    @Environment(\.diffFontFamily) private var fontFamily

    public init(diff: FileDiff) {
        self.diff = diff
    }

    public func makeCoordinator() -> SideBySideCoordinator {
        SideBySideCoordinator()
    }

    public func makeNSView(context: Context) -> NSSplitView {
        let split = NSSplitView()
        split.isVertical = true
        split.dividerStyle = .thin
        split.delegate = context.coordinator

        let left = makeContainer()
        let right = makeContainer()
        (left.textView as? DiffTextView)?.findPartner = right.textView as? DiffTextView
        (right.textView as? DiffTextView)?.findPartner = left.textView as? DiffTextView
        // We pre-wrap rows in Rust via `wrapSbsRows` so each visual row is shorter
        // than the pane's column count. The NSTextContainer therefore should NOT
        // wrap on its own — that would re-wrap the pre-wrapped row and desync the panes.
        left.wrapsText = false
        right.wrapsText = false
        split.addSubview(left)
        split.addSubview(right)

        context.coordinator.leftContainer = left
        context.coordinator.rightContainer = right
        context.coordinator.startObserving()

        return split
    }

    public func updateNSView(_ split: NSSplitView, context: Context) {
        let font = NSFont(name: fontFamily, size: fontSize) ?? .monospacedSystemFont(ofSize: fontSize, weight: .regular)
        let theme = DiffColors(isDark: colorScheme == .dark)
        context.coordinator.diff = diff
        context.coordinator.font = font
        context.coordinator.theme = theme
        context.coordinator.renderIfNeeded(force: true)
    }
}
