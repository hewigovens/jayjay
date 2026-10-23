import JayJayCore
import SwiftUI

struct FontFamilyGroup: Identifiable {
    let title: String
    let families: [AppSettings.FontFamily]

    var id: String {
        title
    }
}

struct FontFamilyPicker: View {
    @Binding var selection: AppSettings.FontFamily
    let systemTitle: String
    var groups: [FontFamilyGroup] = []

    var body: some View {
        Picker(selection: $selection) {
            Text(systemTitle).tag(AppSettings.FontFamily.system)
            ForEach(groups.filter { !$0.families.isEmpty }) { group in
                Section(group.title) {
                    ForEach(group.families) { Text($0.rawValue).tag($0) }
                }
            }
            if selection != .system, !groups.contains(where: { $0.families.contains(selection) }) {
                Text(selection.rawValue).tag(selection)
            }
        } label: {
            SettingsLabel(Localized.Settings.fontFamily, icon: "textformat")
            if !selection.isInstalled {
                Text(Localized.Settings.fontNotInstalled(family: selection.rawValue, fallback: systemTitle))
            }
        }
    }
}
