import SwiftUI

struct DescriptionBodyPreview: NSViewRepresentable {
    let text: String
    let collapsedHeight: CGFloat
    let expandedHeight: CGFloat
    let expanded: Bool
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    func makeNSView(context _: Context) -> DescriptionScrollView {
        DescriptionScrollView()
    }

    func updateNSView(_ view: DescriptionScrollView, context _: Context) {
        view.setDescription(text, font: fontFamily.scaledNSFont(12, baseSize: baseFontSize))
        view.setExpanded(expanded)
    }

    func sizeThatFits(_ proposal: ProposedViewSize, nsView: DescriptionScrollView, context _: Context) -> CGSize? {
        guard let width = proposal.width, width > 0, width.isFinite else { return nil }
        let height = nsView.contentHeight(for: width)
        let cap = expanded ? expandedHeight : collapsedHeight
        return CGSize(width: width, height: min(height, nsView.wholeLines(fitting: cap)))
    }
}
