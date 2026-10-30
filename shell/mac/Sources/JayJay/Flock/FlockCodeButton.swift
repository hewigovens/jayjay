import JayJayCore
import SwiftUI

struct FlockCodeButton: View {
    @Environment(FlockModel.self) private var flock
    @State private var redeeming = false

    var body: some View {
        Button(Localized.Settings.flockCodeEnter) { redeeming = true }
            .accessibilityIdentifier(AID.Settings.flockCode)
            .sheet(isPresented: $redeeming) {
                FlockCodeSheet(flock: flock, onDismiss: { redeeming = false })
            }
    }
}
