import JayJayCore
import SwiftUI

struct SettingsAppearancePage: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Form {
            Section {
                Picker(selection: Binding(
                    get: { settings.appearanceMode },
                    set: { settings.appearanceMode = $0 }
                )) {
                    ForEach(AppSettings.AppearanceMode.allCases) { mode in
                        Text(mode.title).tag(mode)
                    }
                } label: {
                    SettingsLabel(Localized.Settings.pageAppearance, icon: "circle.lefthalf.filled")
                }
                .pickerStyle(.segmented)
            }

            DiffPaletteSection()
        }
        .formStyle(.grouped)
    }
}
