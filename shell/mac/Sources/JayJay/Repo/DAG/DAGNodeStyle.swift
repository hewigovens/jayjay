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
        DAGNodeStyle(
            isWorkingCopy: change.isWorkingCopy,
            isImmutable: change.isImmutable,
            hasConflict: change.hasConflict,
            isEmpty: change.isEmpty,
            hasBookmark: !change.bookmarks.isEmpty
        )
    }

    init(isWorkingCopy: Bool, isImmutable: Bool, hasConflict: Bool, isEmpty: Bool, hasBookmark: Bool) {
        shape = isImmutable ? .diamond : .circle
        radius = Self.radius(for: shape, hasBookmark: hasBookmark)
        fill = if isWorkingCopy {
            .filled(.accentColor)
        } else if hasConflict {
            .filled(.red)
        } else if isEmpty {
            .outlined(.secondary, lineWidth: 1.5)
        } else if hasBookmark {
            .outlined(.accentColor, lineWidth: 1.8)
        } else {
            .filled(.secondary)
        }
    }

    static func radius(for shape: Shape, hasBookmark: Bool) -> CGFloat {
        let circleRadius: CGFloat = hasBookmark ? nodeRadius + 1 : nodeRadius
        return shape == .diamond ? circleRadius * diamondRadiusScale : circleRadius
    }

    func draw(_ path: Path, in ctx: GraphicsContext) {
        switch fill {
            case let .filled(color):
                ctx.fill(path, with: .color(color))
            case let .outlined(color, lineWidth):
                ctx.stroke(path, with: .color(color), style: StrokeStyle(lineWidth: lineWidth))
        }
    }
}
