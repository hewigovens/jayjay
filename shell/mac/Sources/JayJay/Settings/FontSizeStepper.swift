import JayJayCore
import SwiftUI

struct FontSizeStepper: View {
    @Binding var value: Double
    let id: String

    var body: some View {
        HStack {
            SettingsLabel(Localized.Settings.fontSize, icon: "textformat.size")
            Spacer()
            Text(Localized.Settings.fontPoints(size: Int(value)))
                .foregroundStyle(.secondary)
                .monospacedDigit()
                .accessibilityIdentifier(id)
            Stepper("", value: $value, in: AppSettings.fontSizeRange, step: 1)
                .labelsHidden()
                .controlSize(.small)
        }
    }
}
