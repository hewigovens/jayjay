import JayJayCore
import SwiftUI

extension ChangeDetailView {
    var reviewMode: ReviewMode {
        reviewStore.reviewMode(for: detail.info)
    }

    /// Notes follow `showsReviewControls`; marks, Space, and auto-advance also need the change in review mode.
    var showsReviewMarks: Bool {
        showsReviewControls && reviewMode.showsMarks
    }

    @ViewBuilder var reviewModeToggle: some View {
        let mode = reviewMode
        if showsReviewControls, mode.isSwitchable {
            let isOn = mode == .on
            let toggle = { reviewStore.setReviewMode(!isOn, for: detail.info) }
            Toggle(isOn: Binding(get: { isOn }, set: { _ in toggle() })) {
                Image(systemName: isOn ? "checkmark.circle.fill" : "checkmark.circle")
                    .foregroundStyle(isOn ? Color.accentColor : .secondary)
                    .jayjayFont(14)
            }
            .toggleStyle(.button)
            .buttonStyle(.plain)
            .keyboardFocusStop(.reviewMode, action: toggle)
            .help(isOn ? Localized.Repo.turnOffReviewMode : Localized.Repo.turnOnReviewMode)
            .accessibilityLabel(Localized.Menu.reviewMode)
            .accessibilityIdentifier(AID.FileList.reviewMode)
        }
    }
}
