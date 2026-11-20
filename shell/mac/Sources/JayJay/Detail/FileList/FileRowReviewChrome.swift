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
            case .hidden, .unreviewed: "square"
            case .partial: "minus.square"
            case .reviewed: "checkmark.square.fill"
            case .changedSinceReview: "square.lefthalf.filled"
        }
    }

    var tint: Color {
        switch self {
            case .hidden: .clear
            case .unreviewed: Color.secondary.opacity(0.4)
            case .partial: Color.secondary
            case .reviewed: .green
            case .changedSinceReview: .green
        }
    }

    var accessibilityLabel: String {
        switch self {
            case .hidden: ""
            case .unreviewed: Localized.Repo.unreviewed
            case .partial: Localized.Repo.partiallyReviewed
            case .reviewed: Localized.Repo.reviewed
            case .changedSinceReview: Localized.Repo.changedSinceReview
        }
    }
}
