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

            Section {
                Picker(selection: Binding(
                    get: { settings.language },
                    set: { settings.language = $0 }
                )) {
                    Text(Localized.Settings.languageSystem).tag(String?.none)
                    ForEach(availableLanguages(), id: \.tag) { language in
                        Text(language.name).tag(Optional(language.tag))
                    }
                } label: {
                    SettingsLabel(Localized.Settings.language, icon: "globe")
                }
            } footer: {
                HStack {
                    Text(Localized.Settings.languageDetail)
                    Spacer()
                    if settings.language != settings.launchLanguage {
                        Button(Localized.Settings.languageRelaunch) { NSApp.relaunch() }
                    }
                }
            }

            DiffPaletteSection()
        }
        .formStyle(.grouped)
    }
}
