import AppKit

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
                "Reviewed"
            case .unreviewed:
                "Unreviewed"
            case .changedSinceReview:
                "Changed since review"
        }
    }
}
