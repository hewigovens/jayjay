import JayJayCore
import SwiftUI

enum DAGLinkComponent: Equatable {
    struct PathGeometry {
        let x: CGFloat
        let topY: CGFloat
        let centerY: CGFloat
        let bottomY: CGFloat
        let halfPitch: CGFloat
        let cornerRadius: CGFloat
    }

    case vertical(DagEdgeKind)
    case horizontal(DagEdgeKind)
    case leftFork(DagEdgeKind)
    case rightFork(DagEdgeKind)
    case leftMerge(DagEdgeKind)
    case rightMerge(DagEdgeKind)

    var edgeKind: DagEdgeKind {
        switch self {
            case let .vertical(kind), let .horizontal(kind), let .leftFork(kind),
                 let .rightFork(kind), let .leftMerge(kind), let .rightMerge(kind):
                kind
        }
    }

    func path(in geometry: PathGeometry) -> Path {
        let x = geometry.x
        let topY = geometry.topY
        let centerY = geometry.centerY
        let bottomY = geometry.bottomY
        let halfPitch = geometry.halfPitch
        let radius = min(geometry.cornerRadius, halfPitch, centerY - topY, bottomY - centerY)
        return Path { path in
            switch self {
                case .vertical:
                    path.move(to: CGPoint(x: x, y: topY))
                    path.addLine(to: CGPoint(x: x, y: bottomY))
                case .horizontal:
                    path.move(to: CGPoint(x: x - halfPitch, y: centerY))
                    path.addLine(to: CGPoint(x: x + halfPitch, y: centerY))
                case .leftFork:
                    path.move(to: CGPoint(x: x - halfPitch, y: centerY))
                    path.addLine(to: CGPoint(x: x - radius, y: centerY))
                    path.addQuadCurve(
                        to: CGPoint(x: x, y: centerY + radius),
                        control: CGPoint(x: x, y: centerY)
                    )
                    path.addLine(to: CGPoint(x: x, y: bottomY))
                case .rightFork:
                    path.move(to: CGPoint(x: x + halfPitch, y: centerY))
                    path.addLine(to: CGPoint(x: x + radius, y: centerY))
                    path.addQuadCurve(
                        to: CGPoint(x: x, y: centerY + radius),
                        control: CGPoint(x: x, y: centerY)
                    )
                    path.addLine(to: CGPoint(x: x, y: bottomY))
                case .leftMerge:
                    path.move(to: CGPoint(x: x, y: topY))
                    path.addLine(to: CGPoint(x: x, y: centerY - radius))
                    path.addQuadCurve(
                        to: CGPoint(x: x - radius, y: centerY),
                        control: CGPoint(x: x, y: centerY)
                    )
                    path.addLine(to: CGPoint(x: x - halfPitch, y: centerY))
                case .rightMerge:
                    path.move(to: CGPoint(x: x, y: topY))
                    path.addLine(to: CGPoint(x: x, y: centerY - radius))
                    path.addQuadCurve(
                        to: CGPoint(x: x + radius, y: centerY),
                        control: CGPoint(x: x, y: centerY)
                    )
                    path.addLine(to: CGPoint(x: x + halfPitch, y: centerY))
            }
        }
    }
}

extension DagLinkCell {
    var components: [DAGLinkComponent] {
        [
            vertical.map(DAGLinkComponent.vertical),
            horizontal.map(DAGLinkComponent.horizontal),
            leftFork.map(DAGLinkComponent.leftFork),
            rightFork.map(DAGLinkComponent.rightFork),
            leftMerge.map(DAGLinkComponent.leftMerge),
            rightMerge.map(DAGLinkComponent.rightMerge)
        ].compactMap(\.self)
    }
}
