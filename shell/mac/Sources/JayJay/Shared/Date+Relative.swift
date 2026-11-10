import Foundation
import JayJayCore

extension Date {
    private static let relativeFormatter = RelativeDateTimeFormatter()
    private static let abbreviatedRelativeFormatter: RelativeDateTimeFormatter = {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter
    }()

    /// Floored to whole minutes: a per-second count on fresh changes is distracting, and a clock-skewed future timestamp then reads as "1 minute ago".
    static func relativeLabel(millis: Int64) -> String {
        let date = Date(timeIntervalSince1970: Double(millis) / 1000)
        let now = Date()
        relativeFormatter.locale = Localized.locale
        return relativeFormatter.localizedString(for: min(date, now.addingTimeInterval(-60)), relativeTo: now)
    }

    static func abbreviatedRelativeLabel(millis: Int64) -> String {
        abbreviatedRelativeFormatter.locale = Localized.locale
        return abbreviatedRelativeFormatter.localizedString(for: Date(timeIntervalSince1970: Double(millis) / 1000), relativeTo: Date())
    }
}
