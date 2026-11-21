import SwiftUI

public struct ReviewStateIcon: View {
    var state: ReviewState

    public init(state: ReviewState) {
        self.state = state
    }

    public var body: some View {
        Image(systemName: symbol)
            .foregroundStyle(state == .unreviewed ? AnyShapeStyle(.tertiary) : AnyShapeStyle(ReviewColors.reviewed))
            .accessibilityHidden(true)
    }

    private var symbol: String {
        switch state {
            case .unreviewed: "square"
            case .partial: "minus.square"
            case .changedSinceReview: "square.lefthalf.filled"
            case .reviewed: "checkmark.square.fill"
        }
    }
}
