import JayJayCore
import SwiftUI

public struct ReviewMarkButton: View {
    var state: ReviewState
    var action: () -> Void

    public init(state: ReviewState, action: @escaping () -> Void) {
        self.state = state
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            ReviewStateIcon(state: state)
                .frame(minWidth: 44, minHeight: 44)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(Localized.Web.markReviewed)
        .accessibilityValue(state.label)
        .accessibilityAddTraits(state == .reviewed ? [.isToggle, .isSelected] : .isToggle)
    }
}
