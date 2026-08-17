import Foundation

extension Date {
    private static let relativeFormatter = RelativeDateTimeFormatter()

    /// Floored to whole minutes: a per-second count on fresh changes is distracting, and a clock-skewed future timestamp then reads as "1 minute ago".
    static func relativeLabel(millis: Int64) -> String {
        let date = Date(timeIntervalSince1970: Double(millis) / 1000)
        let now = Date()
        return relativeFormatter.localizedString(for: min(date, now.addingTimeInterval(-60)), relativeTo: now)
    }
}
