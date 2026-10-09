import SwiftUI

struct FontSizeStepper: View {
    @Binding var value: Double
    let id: String

    var body: some View {
        HStack {
            SettingsLabel("Size", icon: "textformat.size")
            Spacer()
            Text("\(Int(value)) pt")
                .foregroundStyle(.secondary)
                .monospacedDigit()
                .accessibilityIdentifier(id)
            Stepper("", value: $value, in: AppSettings.fontSizeRange, step: 1)
                .labelsHidden()
                .controlSize(.small)
        }
    }
}
