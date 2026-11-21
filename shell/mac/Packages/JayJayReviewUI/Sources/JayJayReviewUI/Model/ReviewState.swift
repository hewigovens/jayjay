import JayJayCore

public enum ReviewState: String, Codable, Sendable {
    case reviewed, unreviewed, partial
    case changedSinceReview = "changed_since_review"

    public var label: String {
        switch self {
            case .reviewed: Localized.Web.reviewed
            case .unreviewed: Localized.Web.unreviewed
            case .partial: Localized.Web.partial
            case .changedSinceReview: Localized.Web.changedSinceReview
        }
    }
}
