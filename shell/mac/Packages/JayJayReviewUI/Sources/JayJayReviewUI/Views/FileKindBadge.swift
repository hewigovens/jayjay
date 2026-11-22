import SwiftUI

public struct FileKindBadge: View {
    var kind: FileKind

    public init(kind: FileKind) {
        self.kind = kind
    }

    public var body: some View {
        let tint = ReviewColors.kind(kind)
        KindGlyph(kind: kind)
            .stroke(tint, style: StrokeStyle(lineWidth: 1.6, lineCap: .round, lineJoin: .round))
            .frame(width: 16, height: 16)
            .background(tint.opacity(0.15), in: RoundedRectangle(cornerRadius: 4, style: .continuous))
            .accessibilityElement()
            .accessibilityLabel(kind.label)
    }
}

/// The badge's glyph, drawn in review-web's 16-unit box.
private struct KindGlyph: Shape {
    var kind: FileKind

    func path(in rect: CGRect) -> Path {
        let scale = min(rect.width, rect.height) / 16
        func p(_ x: CGFloat, _ y: CGFloat) -> CGPoint {
            CGPoint(x: rect.minX + x * scale, y: rect.minY + y * scale)
        }
        var path = Path()
        func line(_ points: (CGFloat, CGFloat)...) {
            guard let first = points.first else { return }
            path.move(to: p(first.0, first.1))
            for point in points.dropFirst() {
                path.addLine(to: p(point.0, point.1))
            }
        }
        switch kind {
            case .added:
                line((8, 4.5), (8, 11.5))
                line((4.5, 8), (11.5, 8))
            case .removed:
                line((4.5, 8), (11.5, 8))
            case .modified:
                line((8, 3.5), (8, 8.5))
                line((5.5, 6), (10.5, 6))
                line((5.5, 11.5), (10.5, 11.5))
            case .renamed:
                line((4.5, 8), (11.5, 8))
                line((8.5, 5), (11.5, 8), (8.5, 11))
            case .copied:
                line((6.5, 6.5), (11.5, 6.5), (11.5, 11.5), (6.5, 11.5), (6.5, 6.5))
                line((4.5, 9.5), (4.5, 4.5), (9.5, 4.5))
        }
        return path
    }
}
