import AppKit
import JayJayCore
import SwiftUI

public struct SideBySideRepresentable: NSViewRepresentable {
    public typealias Coordinator = SideBySideCoordinator

    public let diff: FileDiff
    public var onExpandContext: ((DiffContextExpansionRequest) -> Void)?
    public var resetSelectionGeneration: UInt64
    public var revealFeedback: DiffContextRevealFeedback?

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.diffFontSize) private var fontSize
    @Environment(\.diffFontFamily) private var fontFamily
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    public init(
        diff: FileDiff,
        onExpandContext: ((DiffContextExpansionRequest) -> Void)? = nil,
        resetSelectionGeneration: UInt64 = 0,
        revealFeedback: DiffContextRevealFeedback? = nil
    ) {
        self.diff = diff
        self.onExpandContext = onExpandContext
        self.resetSelectionGeneration = resetSelectionGeneration
        self.revealFeedback = revealFeedback
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
        left.textView.delegate = context.coordinator
        right.textView.delegate = context.coordinator
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
        context.coordinator.onExpandContext = onExpandContext
        context.coordinator.revealFeedback = revealFeedback
        context.coordinator.reduceMotion = reduceMotion
        context.coordinator.applySelectionResetGeneration(resetSelectionGeneration)
        context.coordinator.renderIfNeeded(force: true)
    }
}
