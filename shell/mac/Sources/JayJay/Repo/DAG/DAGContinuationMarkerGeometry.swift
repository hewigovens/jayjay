import JayJayCore
import SwiftUI

struct DAGContinuationMarkerGeometry {
    private static let boundaryInset: CGFloat = 2
    private static let arrowheadHalfWidth: CGFloat = 2.5
    private static let arrowheadDepth: CGFloat = 4

    let shaftStart: CGPoint
    let tip: CGPoint
    let arrowheadLeft: CGPoint
    let arrowheadRight: CGPoint

    init(
        direction: DagContinuationDirection,
        x: CGFloat,
        rowHeight: CGFloat,
        nodeY: CGFloat,
        nodeRadius: CGFloat
    ) {
        let pointsTowardTop = direction == .incoming
        let tipY = pointsTowardTop ? Self.boundaryInset : rowHeight - Self.boundaryInset
        let arrowheadBaseY = tipY + (pointsTowardTop ? Self.arrowheadDepth : -Self.arrowheadDepth)
        shaftStart = CGPoint(
            x: x,
            y: pointsTowardTop ? nodeY - nodeRadius : nodeY + nodeRadius
        )
        tip = CGPoint(x: x, y: tipY)
        arrowheadLeft = CGPoint(x: x - Self.arrowheadHalfWidth, y: arrowheadBaseY)
        arrowheadRight = CGPoint(x: x + Self.arrowheadHalfWidth, y: arrowheadBaseY)
    }

    var shaftPath: Path {
        Path { path in
            path.move(to: shaftStart)
            path.addLine(to: tip)
        }
    }

    var arrowheadPath: Path {
        Path { path in
            path.move(to: arrowheadLeft)
            path.addLine(to: tip)
            path.addLine(to: arrowheadRight)
        }
    }

    var points: [CGPoint] {
        [shaftStart, tip, arrowheadLeft, arrowheadRight]
    }
}
