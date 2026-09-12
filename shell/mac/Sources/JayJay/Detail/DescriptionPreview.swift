import AppKit
import SwiftUI

struct DescriptionPreview: NSViewRepresentable {
    let description: String
    let maximumHeight: CGFloat
    let expanded: Bool
    let onEdit: (() -> Void)?
    let onOverflowChanged: (Bool) -> Void
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> DescriptionScrollView {
        DescriptionScrollView()
    }

    func updateNSView(_ view: DescriptionScrollView, context: Context) {
        context.coordinator.onOverflowChanged = onOverflowChanged
        view.onEdit = onEdit
        view.setDescription(
            description,
            font: fontFamily.nsFont(size: baseFontSize),
            titleFont: .systemFont(ofSize: 14 * baseFontSize / 12, weight: .semibold)
        )
        view.setExpanded(expanded)
    }

    func sizeThatFits(_ proposal: ProposedViewSize, nsView: DescriptionScrollView, context: Context) -> CGSize? {
        guard let width = proposal.width, width > 0, width.isFinite else { return nil }
        let height = nsView.contentHeight(for: width)
        context.coordinator.reportOverflow(height > maximumHeight)
        let limit = maximumHeight * (expanded ? 4 : 1)
        return CGSize(width: width, height: min(height, max(0, limit)))
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
