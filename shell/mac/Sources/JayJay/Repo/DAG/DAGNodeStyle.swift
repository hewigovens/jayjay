import JayJayCore
import SwiftUI

struct DAGNodeStyle {
    enum Shape: Equatable {
        case circle
        case diamond
    }

    enum Fill {
        case filled(Color)
        case outlined(Color, lineWidth: CGFloat)
    }

    private static let diamondRadiusScale = (CGFloat.pi / 2).squareRoot()

    let shape: Shape
    let radius: CGFloat
    let fill: Fill

    func path(in rect: CGRect) -> Path {
        switch shape {
            case .circle:
                Path(ellipseIn: rect)
            case .diamond:
                Path { p in
                    let cx = rect.midX
                    let cy = rect.midY
                    p.move(to: CGPoint(x: cx, y: rect.minY))
                    p.addLine(to: CGPoint(x: rect.maxX, y: cy))
                    p.addLine(to: CGPoint(x: cx, y: rect.maxY))
                    p.addLine(to: CGPoint(x: rect.minX, y: cy))
                    p.closeSubpath()
                }
        }
    }

    static func resolve(change: ChangeInfo) -> DAGNodeStyle {
        let hasBookmark = !change.bookmarks.isEmpty
        let shape: Shape = change.isImmutable ? .diamond : .circle
        let circleRadius: CGFloat = hasBookmark ? nodeRadius + 1 : nodeRadius
        let radius = shape == .diamond ? circleRadius * diamondRadiusScale : circleRadius

        let fill: Fill = if change.isWorkingCopy {
            .filled(.accentColor)
        } else if change.hasConflict {
            .filled(.red)
        } else if change.isEmpty {
            .outlined(.secondary, lineWidth: 1.5)
        } else if hasBookmark {
            .outlined(.accentColor, lineWidth: 1.8)
        } else {
            .filled(.secondary)
        }

        return DAGNodeStyle(shape: shape, radius: radius, fill: fill)
    }
}
