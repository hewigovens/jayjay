import JayJayCore
import SwiftUI

extension DAGRow {
    var graphColumn: some View {
        GeometryReader { geo in
            let myLane = viewModel.layout.lane(for: change.commitId.id)
            let myDisplayLane = viewModel.layout.displayLane(for: myLane)
            let myX = viewModel.layout.xPosition(forDisplayLane: myDisplayLane)
            let hasOverflow = viewModel.layout.hasLaneOverflow(at: viewModel.index)
            let overflowDisplayLane = viewModel.layout.displayLaneCount() - 1
            let passThroughDisplayLanes = Set(
                viewModel.layout.passThroughLaneIndices(at: viewModel.index).map {
                    viewModel.layout.displayLane(for: $0)
                }
            ).sorted()
            let nodeY = dagNodeCenterY
            let height = geo.size.height

            Canvas { ctx, _ in
                let lineColor = Color.secondary.opacity(0.2)
                let edgeColor = Color.secondary.opacity(0.3)
                let laneStroke: (Int) -> StrokeStyle = { displayLane in
                    hasOverflow && displayLane == overflowDisplayLane
                        ? dagOverflowStroke
                        : dagSolidStroke
                }

                for displayLane in passThroughDisplayLanes where displayLane != myDisplayLane {
                    let laneX = viewModel.layout.xPosition(forDisplayLane: displayLane)
                    let path = Path { p in
                        p.move(to: CGPoint(x: laneX, y: 0))
                        p.addLine(to: CGPoint(x: laneX, y: height))
                    }
                    ctx.stroke(path, with: .color(lineColor), style: laneStroke(displayLane))
                }

                // Top stub: connect down from the row above when the lane continues.
                if viewModel.index > 0 {
                    let prevActive = viewModel.layout.activeLaneIndices(at: viewModel.index - 1)
                    if prevActive.contains(myLane) {
                        let path = Path { p in
                            p.move(to: CGPoint(x: myX, y: 0))
                            p.addLine(to: CGPoint(x: myX, y: nodeY - nodeRadius))
                        }
                        ctx.stroke(path, with: .color(lineColor), style: laneStroke(myDisplayLane))
                    }
                }

                // Bottom stub for non-tail nodes on a forking lane.
                let hasSameLaneParent = viewModel.entry.edges.contains { edge in
                    edge.edgeType != .missing && viewModel.layout.lane(for: edge.target) == myLane
                }
                if !hasSameLaneParent {
                    let nextActive = viewModel.layout.activeLaneIndices(at: viewModel.index + 1)
                    if nextActive.contains(myLane) {
                        let path = Path { p in
                            p.move(to: CGPoint(x: myX, y: nodeY + nodeRadius))
                            p.addLine(to: CGPoint(x: myX, y: height))
                        }
                        ctx.stroke(path, with: .color(lineColor), style: laneStroke(myDisplayLane))
                    }
                }

                for edge in viewModel.entry.edges {
                    if edge.edgeType == .missing {
                        continue
                    }
                    let targetLane = viewModel.layout.lane(for: edge.target)
                    let targetDisplayLane = viewModel.layout.displayLane(for: targetLane)
                    let targetX = viewModel.layout.xPosition(forDisplayLane: targetDisplayLane)

                    let path = Path { p in
                        p.move(to: CGPoint(x: myX, y: nodeY + nodeRadius))
                        if targetDisplayLane == myDisplayLane {
                            p.addLine(to: CGPoint(x: myX, y: height))
                        } else {
                            let midY = nodeY + nodeRadius + (height - nodeY - nodeRadius) * 0.4
                            p.addLine(to: CGPoint(x: myX, y: midY))
                            p.addQuadCurve(
                                to: CGPoint(x: targetX, y: height),
                                control: CGPoint(x: targetX, y: midY)
                            )
                        }
                    }
                    let style: StrokeStyle = if edge.edgeType == .indirect {
                        dagIndirectEdgeStroke
                    } else if hasOverflow, myDisplayLane == overflowDisplayLane || targetDisplayLane == overflowDisplayLane {
                        dagOverflowStroke
                    } else {
                        dagSolidStroke
                    }
                    ctx.stroke(path, with: .color(edgeColor), style: style)
                }

                let style = DAGNodeStyle.resolve(change: change)
                let nodeRect = CGRect(
                    x: myX - style.radius,
                    y: nodeY - style.radius,
                    width: style.radius * 2,
                    height: style.radius * 2
                )
                let nodePath = style.path(in: nodeRect)
                switch style.fill {
                    case let .filled(color):
                        ctx.fill(nodePath, with: .color(color))
                    case let .outlined(color, lineWidth):
                        ctx.stroke(nodePath, with: .color(color), style: StrokeStyle(lineWidth: lineWidth))
                }

                if viewModel.isRebaseCandidate {
                    ctx.stroke(
                        nodePath,
                        with: .color(.accentColor.opacity(viewModel.isRebaseHoverTarget ? 1 : 0.55)),
                        style: StrokeStyle(lineWidth: viewModel.isRebaseHoverTarget ? 2.5 : 1.4)
                    )
                    if viewModel.isRebaseHoverTarget {
                        let ringRect = nodeRect.insetBy(dx: -4, dy: -4)
                        ctx.stroke(
                            style.path(in: ringRect),
                            with: .color(.accentColor.opacity(0.45)),
                            style: StrokeStyle(lineWidth: 2)
                        )
                    }
                } else if viewModel.isRebaseSource {
                    ctx.stroke(
                        nodePath,
                        with: .color(.accentColor.opacity(0.75)),
                        style: StrokeStyle(lineWidth: 2)
                    )
                    if viewModel.isRebaseArmed {
                        let ringRect = nodeRect.insetBy(dx: -3, dy: -3)
                        ctx.stroke(
                            style.path(in: ringRect),
                            with: .color(.accentColor.opacity(0.35)),
                            style: StrokeStyle(lineWidth: 1.5, dash: [3, 3])
                        )
                    }
                }
            }
            .clipped()
        }
    }
}
