import SwiftUI

struct FlowLayout: Layout {
    struct Plan {
        let frames: [CGRect]
        let size: CGSize
    }

    var spacing: CGFloat = 6
    var lineSpacing: CGFloat = 6

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout Void) -> CGSize {
        plan(sizes: subviews.map { $0.sizeThatFits(.unspecified) }, maxWidth: proposal.width ?? .infinity).size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout Void) {
        let frames = plan(sizes: subviews.map { $0.sizeThatFits(.unspecified) }, maxWidth: bounds.width).frames
        for (subview, frame) in zip(subviews, frames) {
            subview.place(
                at: CGPoint(x: bounds.minX + frame.minX, y: bounds.minY + frame.minY),
                proposal: ProposedViewSize(frame.size)
            )
        }
    }

    func plan(sizes: [CGSize], maxWidth: CGFloat) -> Plan {
        var frames: [CGRect] = []
        var origin = CGPoint.zero
        var lineHeight: CGFloat = 0
        var width: CGFloat = 0
        for size in sizes {
            if origin.x > 0, origin.x + size.width > maxWidth {
                origin = CGPoint(x: 0, y: origin.y + lineHeight + lineSpacing)
                lineHeight = 0
            }
            frames.append(CGRect(origin: origin, size: size))
            origin.x += size.width + spacing
            lineHeight = max(lineHeight, size.height)
            width = max(width, origin.x - spacing)
        }
        return Plan(frames: frames, size: CGSize(width: width, height: origin.y + lineHeight))
    }
}
