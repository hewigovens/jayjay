import SwiftUI

struct FlockUnlockButton: View {
    @Environment(FlockModel.self) private var flock
    let title: String
    @State private var activating = false

    var body: some View {
        Button(title) { activating = true }
            .accessibilityIdentifier(AID.Settings.flockUnlock)
            .sheet(isPresented: $activating) {
                FlockActivationSheet(flock: flock, onDismiss: { activating = false })
            }
    }
}
