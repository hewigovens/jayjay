import CoreGraphics
import JayJayCore

struct OverviewPlacement {
    struct Band {
        let base: OverviewBase
        let y: CGFloat
        let lastX: CGFloat
    }

    struct Lane: Identifiable {
        let laneIndex: Int
        let x: CGFloat
        let bandY: CGFloat
        let count: Int

        var id: Int {
            laneIndex
        }

        func nodeY(_ offset: Int) -> CGFloat {
            bandY - CGFloat(count - offset) * OverviewGeometry.rowHeight
        }

        var cardCenterY: CGFloat {
            nodeY(0) - OverviewGeometry.rowHeight / 2 - OverviewGeometry.cardGap - OverviewGeometry.cardHeight / 2
        }

        /// Includes the stem down to the band line.
        var frame: CGRect {
            let top = cardCenterY - OverviewGeometry.cardHeight / 2
            return CGRect(x: x, y: top, width: OverviewGeometry.columnWidth, height: bandY - top)
        }

        func rowOffsets(in rect: CGRect) -> [Int] {
            (0 ..< count).filter { offset in
                let y = nodeY(offset)
                return y + OverviewGeometry.rowHeight / 2 > rect.minY && y - OverviewGeometry.rowHeight / 2 < rect.maxY
            }
        }
    }

    let bands: [Band]
    let lanes: [Lane]
    let size: CGSize

    func lanes(in rect: CGRect) -> [Lane] {
        lanes.filter { $0.frame.intersects(rect) }
    }

    func bands(in rect: CGRect) -> [Band] {
        let reach = OverviewGeometry.rowHeight + 8
        return bands.filter { $0.y + reach > rect.minY && $0.y - reach < rect.maxY }
    }

    /// Snapping to tiles re-renders only when scrolling crosses a tile edge.
    static func tiled(_ rect: CGRect) -> CGRect {
        let tile: CGFloat = 512
        let minX = ((rect.minX - tile) / tile).rounded(.down) * tile
        let minY = ((rect.minY - tile) / tile).rounded(.down) * tile
        let maxX = ((rect.maxX + tile) / tile).rounded(.up) * tile
        let maxY = ((rect.maxY + tile) / tile).rounded(.up) * tile
        return CGRect(x: minX, y: minY, width: maxX - minX, height: maxY - minY)
    }

    init(lanes allLanes: [OverviewLane], groups: [OverviewGroup]) {
        typealias Geo = OverviewGeometry
        var bands: [Band] = []
        var lanes: [Lane] = []
        var widestGroup = 0
        var previousBandY: CGFloat?
        for group in groups {
            let counts = group.lanes.map { allLanes[Int($0)].changes.count }
            let top = previousBandY.map { $0 + Geo.bandSpacing } ?? Geo.topPadding
            let bandY = top + Geo.cardHeight + Geo.cardGap + Geo.rowHeight * (CGFloat(counts.max() ?? 0) + 0.5)
            previousBandY = bandY
            var lastX = Geo.spineX
            for (column, laneIndex) in group.lanes.enumerated() {
                let x = Geo.gutterWidth + CGFloat(column) * (Geo.columnWidth + Geo.columnGap)
                lanes.append(Lane(laneIndex: Int(laneIndex), x: x, bandY: bandY, count: counts[column]))
                lastX = x + Geo.nodeInset
            }
            widestGroup = max(widestGroup, group.lanes.count)
            bands.append(Band(base: group.base, y: bandY, lastX: lastX))
        }
        self.bands = bands
        self.lanes = lanes
        size = CGSize(
            width: Geo.gutterWidth + CGFloat(widestGroup) * (Geo.columnWidth + Geo.columnGap) + 40,
            height: (bands.last?.y ?? 0) + 40
        )
    }
}
