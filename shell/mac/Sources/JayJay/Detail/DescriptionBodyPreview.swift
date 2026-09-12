import SwiftUI

struct DescriptionBodyPreview: NSViewRepresentable {
    let text: String
    let maximumHeight: CGFloat
    let expanded: Bool
    let onOverflowChanged: (Bool) -> Void
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context _: Context) -> DescriptionScrollView {
        DescriptionScrollView()
    }

    func updateNSView(_ view: DescriptionScrollView, context: Context) {
        context.coordinator.onOverflowChanged = onOverflowChanged
        view.setDescription(text, font: fontFamily.nsFont(size: baseFontSize))
        view.setExpanded(expanded)
    }

    func sizeThatFits(_ proposal: ProposedViewSize, nsView: DescriptionScrollView, context: Context) -> CGSize? {
        guard let width = proposal.width, width > 0, width.isFinite else { return nil }
        let height = nsView.contentHeight(for: width)
        context.coordinator.reportOverflow(height > maximumHeight)
        return CGSize(width: width, height: min(height, maximumHeight * (expanded ? 4 : 1)))
    }

    final class Coordinator {
        var onOverflowChanged: (Bool) -> Void = { _ in }
        private var overflow = false

        func reportOverflow(_ value: Bool) {
            guard overflow != value else { return }
            overflow = value
            // Measuring runs during layout; publish only threshold crossings after that pass.
            DispatchQueue.main.async { [weak self] in
                guard let self, overflow == value else { return }
                onOverflowChanged(value)
            }
        }
    }
}
