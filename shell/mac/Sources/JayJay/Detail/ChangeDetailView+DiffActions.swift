import JayJayCore
import SwiftUI

extension ChangeDetailView {
    var diffActions: some View {
        let sideBySide = appSettings.sideBySideDiff
        let stacked = isStacked
        let toggleLayout = { appSettings.sideBySideDiff.toggle() }
        let openDiffEdit = { paneMode = .diffEdit }
        return HStack(spacing: 2) {
            Button(action: toggleLayout) {
                HStack(spacing: 4) {
                    Image(systemName: sideBySide ? "rectangle.split.2x1" : "text.justify")
                    Text(sideBySide ? Localized.Diff.sideBySide : Localized.Diff.unified)
                }
            }
            .disabled(stacked)
            .keyboardFocusStop(.diffLayout, isAvailable: !stacked, action: toggleLayout)
            .help(stacked ? Localized.Diff.sideBySideUnavailable : sideBySide ? Localized.Diff.switchToUnified : Localized.Diff.switchToSideBySide)
            .accessibilityIdentifier(AID.Diff.layoutToggle)
            if canEnterDiffEdit, let hunk = selectedHunk, hunk.projection == nil {
                Button(action: openDiffEdit) {
                    HStack(spacing: 4) {
                        Image(systemName: "square.and.pencil")
                        Text(Localized.Diff.editDiff)
                    }
                }
                .keyboardFocusStop(.editDiff, action: openDiffEdit)
                .help(Localized.Diff.openDedicatedDiffEditMode)
                .accessibilityIdentifier(AID.DiffEdit.open)
            }
        }
        .buttonStyle(HeaderActionButtonStyle())
        .fixedSize()
    }
}
