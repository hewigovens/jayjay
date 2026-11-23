import SwiftUI

public struct ReviewStateIcon: View {
    var state: ReviewState

    public init(state: ReviewState) {
        self.state = state
    }

    public var body: some View {
        Image(systemName: symbol)
            .foregroundStyle(tint)
            .accessibilityHidden(true)
    }

    private var tint: AnyShapeStyle {
        switch state {
            case .unreviewed: AnyShapeStyle(.tertiary)
            case .partial, .reviewed: AnyShapeStyle(ReviewColors.reviewed)
            case .changedSinceReview: AnyShapeStyle(ReviewColors.changed)
        }
    }

    private var symbol: String {
        switch state {
            case .unreviewed: "square"
            case .partial: "minus.square"
            case .changedSinceReview: "dot.square"
            case .reviewed: "checkmark.square"
        }
    }
}
