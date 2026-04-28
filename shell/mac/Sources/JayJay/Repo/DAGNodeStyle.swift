import JayJayCore
import SwiftUI

/// Shape marks trunk bookmarks as diamonds; fill encodes working-copy/conflict/empty/bookmark state.
struct DAGNodeStyle {
    enum Shape {
        case circle
        case diamond
    }

    enum Fill {
        case filled(Color)
        case outlined(Color, lineWidth: CGFloat)
    }

    let shape: Shape
    let radius: CGFloat
    let fill: Fill

    func path(in rect: CGRect) -> Path {
        switch shape {
            case .circle:
                return Path(ellipseIn: rect)
            case .diamond:
                return Path { p in
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
        let isTrunk = change.bookmarks.contains(where: isTrunkBookmark)
        let hasBookmark = !change.bookmarks.isEmpty
        let shape: Shape = isTrunk ? .diamond : .circle
        let radius: CGFloat = (isTrunk || hasBookmark) ? nodeRadius + 1 : nodeRadius

        let fill: Fill
        if change.isWorkingCopy {
            fill = .filled(.accentColor)
        } else if change.hasConflict {
            fill = .filled(.red)
        } else if change.isEmpty {
            fill = .outlined(.secondary.opacity(0.5), lineWidth: 1.5)
        } else if isTrunk || hasBookmark {
            fill = .outlined(.accentColor.opacity(0.85), lineWidth: 1.8)
        } else {
            fill = .filled(.secondary.opacity(0.5))
        }

        return DAGNodeStyle(shape: shape, radius: radius, fill: fill)
    }
}
