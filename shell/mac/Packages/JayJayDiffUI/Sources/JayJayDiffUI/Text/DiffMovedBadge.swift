import AppKit

/// `target` is the other copy's logical line in the view that shows it.
struct DiffMovedBadge: Equatable {
    let line: Int
    let title: String
    let target: Int

    static func badges(rows: [DiffMovedRowInfo], counterparts: [DiffMovedRowInfo]) -> [DiffMovedBadge] {
        var lineOf: [CounterpartKey: Int] = [:]
        for (index, row) in counterparts.enumerated() {
            guard let number = row.number else { continue }
            let key = CounterpartKey(isRemoved: row.style == .removed, number: number)
            if lineOf[key] == nil {
                lineOf[key] = index
            }
        }
        return rows.enumerated().compactMap { index, row in
            guard let moved = row.moved, let title = row.badgeTitle,
                  let target = lineOf[CounterpartKey(isRemoved: row.style != .removed, number: moved.counterpartLine)]
            else { return nil }
            return DiffMovedBadge(line: index, title: title, target: target)
        }
    }
}

private struct CounterpartKey: Hashable {
    let isRemoved: Bool
    let number: UInt32
}
