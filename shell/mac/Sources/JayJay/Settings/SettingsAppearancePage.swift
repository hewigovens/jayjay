import SwiftUI

struct SettingsAppearancePage: View {
    @Environment(AppSettings.self) private var settings
    @State private var userFamilies: [AppSettings.FontFamily] = []

    var body: some View {
        @Bindable var settings = settings
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
                    SettingsLabel("Theme", icon: "circle.lefthalf.filled")
                }
                .pickerStyle(.segmented)
                Toggle(isOn: Binding(
                    get: { settings.tintWindowWithWallpaper },
                    set: { settings.tintWindowWithWallpaper = $0 }
                )) {
                    SettingsLabel("Tint window background with wallpaper color", icon: "rectangle.fill")
                    Text("Allow macOS wallpaper tinting in dark mode. Turn off for a flat, neutral background.")
                }
            }

            Section {
                FontFamilyPicker(
                    selection: $settings.interfaceFontFamily,
                    systemTitle: "System",
                    groups: [FontFamilyGroup(title: "My Fonts", families: userFamilies)]
                )
                FontSizeStepper(value: $settings.interfaceFontSize, id: AID.Settings.interfaceFontSize)
            } header: {
                Text("Interface Font")
            } footer: {
                Text("Press \(AppShortcut.zoomIn.symbol) or \(AppShortcut.zoomOut.symbol) to change the interface and code font sizes, \(AppShortcut.resetZoom.symbol) to reset them.")
            }
        }
        .formStyle(.grouped)
        .task { userFamilies = AppSettings.FontFamily.installedUserFamilies() }
    }
}
