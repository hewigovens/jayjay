import AppKit
import JayJayCore

/// App-agnostic per-group review disposition rendered in the gutter stripe.
public enum DiffGutterHunkReviewState: Equatable, Sendable {
    case reviewed
    case unreviewed
    case changedSinceReview

    public var stripeColor: NSColor {
        switch self {
            case .reviewed:
                .controlAccentColor
            case .unreviewed:
                .selectedTextBackgroundColor
            case .changedSinceReview:
                .systemOrange
        }
    }

    public var accessibilityLabel: String {
        switch self {
            case .reviewed:
                Localized.Repo.reviewed
            case .unreviewed:
                Localized.Repo.unreviewed
            case .changedSinceReview:
                Localized.Repo.changedSinceReview
        }
    }
}
