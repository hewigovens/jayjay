import SwiftUI

struct FontFamilyPicker: View {
    @Binding var selection: AppSettings.FontFamily
    let choices: [AppSettings.FontFamily]
    let myFonts: [AppSettings.FontFamily]
    let systemTitle: String

    var body: some View {
        Picker(selection: $selection) {
            ForEach(choices) { Text(title($0)).tag($0) }
            if !myFonts.isEmpty {
                Section("My Fonts") {
                    ForEach(myFonts) { Text(title($0)).tag($0) }
                }
            }
            if !choices.contains(selection), !myFonts.contains(selection) {
                Text(title(selection)).tag(selection)
            }
        } label: {
            SettingsLabel("Family", icon: "textformat")
            if !selection.isInstalled {
                Text("\(selection.title) is not installed; using \(systemTitle).")
            }
        }
    }

    private func title(_ family: AppSettings.FontFamily) -> String {
        family == .system ? systemTitle : family.title
    }
}
