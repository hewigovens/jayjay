import SwiftUI

/// Status glyphs SF Symbols lacks, on the 24-unit grid of the GPUI status SVGs.
struct StatusGlyphShape: Shape {
    enum Glyph {
        case plusMinus
        case copy
    }

    let glyph: Glyph
    /// The grid's side as a fraction of the overlaid symbol frame.
    let gridFraction: CGFloat

    func path(in rect: CGRect) -> Path {
        var lines = Path()
        let lineWidth: CGFloat
        switch glyph {
            case .plusMinus:
                lines.move(to: CGPoint(x: 6.4, y: 9.5))
                lines.addLine(to: CGPoint(x: 17.6, y: 9.5))
                lines.move(to: CGPoint(x: 12, y: 5))
                lines.addLine(to: CGPoint(x: 12, y: 14))
                lines.move(to: CGPoint(x: 6.4, y: 18.5))
                lines.addLine(to: CGPoint(x: 17.6, y: 18.5))
                lineWidth = 2
            case .copy:
                lines.addRoundedRect(in: CGRect(x: 8.15, y: 6.65, width: 8.7, height: 11.95), cornerSize: CGSize(width: 2, height: 2))
                lines.move(to: CGPoint(x: 5.15, y: 15))
                lines.addLine(to: CGPoint(x: 5.15, y: 5.9))
                lines.addArc(tangent1End: CGPoint(x: 5.15, y: 3.4), tangent2End: CGPoint(x: 7.65, y: 3.4), radius: 2.5)
                lines.addLine(to: CGPoint(x: 14.25, y: 3.4))
                lineWidth = 1.8
        }
        let unit = min(rect.width, rect.height) * gridFraction / 24
        let grid = CGAffineTransform(translationX: rect.midX - 12 * unit, y: rect.midY - 12 * unit).scaledBy(x: unit, y: unit)
        return lines
            .strokedPath(StrokeStyle(lineWidth: lineWidth, lineCap: .round, lineJoin: .round))
            .applying(grid)
    }
}
