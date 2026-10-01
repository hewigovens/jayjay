import ApplicationServices
import Foundation

extension Target {
    private static let rowPrefix = "dag.row."
    private static let lanePrefix = "overview.lane."
    private static let filePrefix = "file.row."
    private static let changeIdLetters = Set("klmnopqrstuvwxyz")

    func resolve(in app: AXElement, rows: RowTable?) throws -> (element: AXElement, point: CGPoint)? {
        let identifier = try row.map { try Self.rowPrefix + String(table(rows).id(for: $0).prefix(12)) } ?? file.map { Self.filePrefix + $0 } ?? id
        let laneHead = try lane.map { try table(rows).id(for: $0) }
        if let laneHead, !laneHead.allSatisfy(Self.changeIdLetters.contains) {
            throw Failure("\(summary) is divergent, so rows.tsv names it by commit id, but lanes are named by change id")
        }
        var skipped = 0
        for (element, summary) in app.descendants() {
            guard matches(summary, of: element, identifier: identifier, laneHead: laneHead),
                  let frame = element.frame, frame.width > 0, frame.height > 0
            else { continue }
            var lineBounds: CGRect?
            if let line {
                guard let bounds = element.bounds(ofLineMatching: line) else { continue }
                lineBounds = bounds
            }
            guard skipped == index else {
                skipped += 1
                continue
            }
            let point = CGPoint(
                x: x.map { frame.minX + $0 } ?? frame.midX,
                y: y.map { frame.minY + $0 } ?? lineBounds?.midY ?? frame.midY
            )
            return (element, point)
        }
        return nil
    }

    private func table(_ rows: RowTable?) throws -> RowTable {
        guard let rows else { throw Failure("\(summary) needs the scene's rows file") }
        return rows
    }

    private func matches(_ summary: AXElement.Summary, of element: AXElement, identifier: String?, laneHead: String?) -> Bool {
        if let identifier, summary.identifier != identifier {
            return false
        }
        if let idPrefix, summary.identifier?.hasPrefix(idPrefix) != true {
            return false
        }
        if let laneHead {
            // A lane carries its head's shortest unique change id prefix, rows.tsv the 12-character one.
            guard let lane = summary.identifier, lane.hasPrefix(Self.lanePrefix), lane.count > Self.lanePrefix.count,
                  laneHead.hasPrefix(lane.dropFirst(Self.lanePrefix.count))
            else { return false }
        }
        if let role, summary.role != role {
            return false
        }
        if let menuItem, summary.role != kAXMenuItemRole || summary.title != menuItem {
            return false
        }
        if let label {
            let texts = [summary.title, summary.description, element.string("AXPlaceholderValue"), element.string(kAXValueAttribute)]
            if !texts.contains(label) {
                return false
            }
        }
        return true
    }
}
