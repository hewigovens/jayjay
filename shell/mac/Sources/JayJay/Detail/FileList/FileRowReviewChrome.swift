import JayJayCore
import SwiftUI

enum FileRowReviewChrome: Equatable {
    case hidden
    case unreviewed
    case partial
    case reviewed
    case changedSinceReview

    static func chrome(showReview: Bool, rollup: ReviewFileRollup) -> Self {
        guard showReview else { return .hidden }
        switch rollup {
        case .unreviewed: return .unreviewed
        case .partial: return .partial
        case .reviewed: return .reviewed
        case .changedSinceReview: return .changedSinceReview
        @unknown default: return .unreviewed
        }
    }

    var systemImage: String {
        switch self {
        case .hidden, .unreviewed: "circle"
        case .partial: "checkmark.circle"
        case .reviewed: "checkmark.circle.fill"
        case .changedSinceReview: "circle.fill"
        }
    }

    var tint: Color {
        switch self {
        case .hidden: .clear
        case .unreviewed: Color.secondary.opacity(0.4)
        case .partial: Color.secondary
        case .reviewed: .green
        case .changedSinceReview: Color.orange.opacity(0.85)
        }
    }

    var accessibilityLabel: String {
        switch self {
        case .hidden: ""
        case .unreviewed: "Unreviewed"
        case .partial: "Partially reviewed"
        case .reviewed: "Reviewed"
        case .changedSinceReview: "Changed since review"
        }
    }
}
