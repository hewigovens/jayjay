import JayJayCore
import SwiftUI

extension DAGRow {
    var graphColumn: some View {
        let geometry = viewModel.geometry
        let row = viewModel.row
        let nodeColumn = Int(row?.nodeColumn ?? 0)
        let myX = geometry.xPosition(forColumn: nodeColumn)
        let nodeY = dagNodeCenterY

        return GeometryReader { geo in
            let height = geo.size.height

            Canvas { ctx, _ in
                let lineColor = Color.secondary.opacity(0.2)
                let edgeColor = Color.secondary.opacity(0.3)

                let linkLine = row?.linkLine
                let linkCenterY = linkLine == nil ? nodeY : nodeY + (height - nodeY) * dagLinkCenterFraction
                let linkBottomY = linkLine == nil ? nodeY : min(height, linkCenterY + dagGraphCornerRadius)

                // The node line is the renderer state above this row's transition band.
                if let nodeLine = row?.nodeLine {
                    for (column, cell) in nodeLine.enumerated() where column != nodeColumn {
                        guard let style = strokeStyle(for: cell) else { continue }
                        let laneX = geometry.xPosition(forColumn: column)
                        let path = Path { p in
                            p.move(to: CGPoint(x: laneX, y: 0))
                            p.addLine(to: CGPoint(x: laneX, y: nodeY))
                        }
                        ctx.stroke(path, with: .color(lineColor), style: style)
                    }
                }

                if let incoming = row?.incoming {
                    let path = Path { p in
                        p.move(to: CGPoint(x: myX, y: 0))
                        p.addLine(to: CGPoint(x: myX, y: nodeY - geometry.nodeRadius))
                    }
                    ctx.stroke(path, with: .color(lineColor), style: strokeStyle(for: incoming))
                }

                if let linkLine {
                    for (column, cell) in linkLine.enumerated() {
                        let x = geometry.xPosition(forColumn: column)
                        for component in cell.components {
                            let path = component.path(in: .init(
                                x: x,
                                topY: cell.isChild && column == nodeColumn ? nodeY + geometry.nodeRadius : nodeY,
                                centerY: linkCenterY,
                                bottomY: linkBottomY,
                                halfPitch: geometry.lanePitch / 2,
                                cornerRadius: dagGraphCornerRadius
                            ))
                            ctx.stroke(path, with: .color(edgeColor), style: strokeStyle(for: component.edgeKind))
                        }
                    }
                }

                // The pad line is the renderer state below the transition band.
                if let padLine = row?.padLine {
                    for (column, cell) in padLine.enumerated() {
                        guard let style = strokeStyle(for: cell) else { continue }
                        let x = geometry.xPosition(forColumn: column)
                        let startY = if linkLine != nil {
                            linkBottomY
                        } else if column == nodeColumn {
                            nodeY + geometry.nodeRadius
                        } else {
                            nodeY
                        }
                        let path = Path { p in
                            p.move(to: CGPoint(x: x, y: startY))
                            p.addLine(to: CGPoint(x: x, y: height))
                        }
                        ctx.stroke(path, with: .color(lineColor), style: style)
                    }
                }

                for column in row?.terminationColumns ?? [] {
                    let terminationX = geometry.xPosition(forColumn: Int(column))
                    let startY = if linkLine != nil {
                        linkBottomY
                    } else if Int(column) == nodeColumn {
                        nodeY + geometry.nodeRadius
                    } else {
                        nodeY
                    }
                    let endY = startY + (height - startY) * 0.55
                    let path = Path { p in
                        p.move(to: CGPoint(x: terminationX, y: startY))
                        p.addLine(to: CGPoint(x: terminationX, y: endY))
                    }
                    ctx.stroke(path, with: .color(edgeColor), style: dagMissingEdgeStroke)
                    let capRect = CGRect(x: terminationX - 1.5, y: endY - 1.5, width: 3, height: 3)
                    ctx.fill(Path(ellipseIn: capRect), with: .color(edgeColor))
                }

                let style = DAGNodeStyle.resolve(change: change, radius: geometry.nodeRadius)
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

    private func strokeStyle(for cell: DagVerticalCell) -> StrokeStyle? {
        switch cell {
            case .empty: nil
            case .direct: dagSolidStroke
            case .indirect: dagIndirectEdgeStroke
        }
    }

    private func strokeStyle(for kind: DagEdgeKind) -> StrokeStyle {
        kind == .indirect ? dagIndirectEdgeStroke : dagSolidStroke
    }
}
