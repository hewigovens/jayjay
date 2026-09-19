import SwiftUI

struct DescriptionBodyPreview: NSViewRepresentable {
    let text: String
    let collapsedHeight: CGFloat
    let expandedHeight: CGFloat
    let expanded: Bool
    @Environment(\.jayjayFontSize) private var baseFontSize

    func makeNSView(context _: Context) -> DescriptionScrollView {
        DescriptionScrollView()
    }

    func updateNSView(_ view: DescriptionScrollView, context _: Context) {
        view.setDescription(text, font: .systemFont(ofSize: baseFontSize))
        view.setExpanded(expanded)
    }

    func sizeThatFits(_ proposal: ProposedViewSize, nsView: DescriptionScrollView, context _: Context) -> CGSize? {
        guard let width = proposal.width, width > 0, width.isFinite else { return nil }
        let height = nsView.contentHeight(for: width)
        let cap = expanded ? expandedHeight : collapsedHeight
        return CGSize(width: width, height: min(height, nsView.wholeLines(fitting: cap)))
    }
}
