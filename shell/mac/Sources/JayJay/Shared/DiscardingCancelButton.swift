import JayJayCore
import SwiftUI

struct DiscardingCancelButton: View {
    let hasChanges: Bool
    let onCancel: () -> Void
    @State private var confirming = false

    var body: some View {
        Button(Localized.Common.cancel) {
            if hasChanges {
                confirming = true
            } else {
                onCancel()
            }
        }
        .keyboardShortcut(.cancelAction)
        .confirmationDialog(Localized.Common.discardChangesTitle, isPresented: $confirming) {
            Button(Localized.Common.discardChanges, role: .destructive, action: onCancel)
            Button(Localized.Common.keepEditing, role: .cancel) {}
        }
    }
}
