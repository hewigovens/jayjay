import SwiftUI

struct SettingsAppearancePage: View {
    @Environment(AppSettings.self) private var settings
    @State private var myFonts: [AppSettings.MonoFont] = []

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
                Picker(selection: Binding(
                    get: { settings.fontFamily },
                    set: { settings.fontFamily = $0 }
                )) {
                    let curated = AppSettings.MonoFont.allCases.filter(\.isInstalled)
                    ForEach(curated) { font in
                        Text(font.title).tag(font)
                    }
                    if !myFonts.isEmpty {
                        Section("My Fonts") {
                            ForEach(myFonts) { font in
                                Text(font.title).tag(font)
                            }
                        }
                    }
                    if !curated.contains(settings.fontFamily), !myFonts.contains(settings.fontFamily) {
                        Text(settings.fontFamily.title).tag(settings.fontFamily)
                    }
                } label: {
                    SettingsLabel("Family", icon: "textformat")
                }
                .task { myFonts = AppSettings.MonoFont.installedMyFonts() }

                HStack {
                    SettingsLabel("Size", icon: "textformat.size")
                    Spacer()
                    Text("\(Int(settings.fontSize))pt")
                        .foregroundStyle(.secondary)
                        .monospacedDigit()
                    Stepper("", value: Binding(
                        get: { settings.fontSize },
                        set: { settings.fontSize = $0 }
                    ), in: AppSettings.fontSizeRange, step: 1)
                        .labelsHidden()
                        .controlSize(.small)
                }
            } header: {
                Text("Font")
            } footer: {
                Text("Press \(AppShortcut.zoomIn.symbol) or \(AppShortcut.zoomOut.symbol) to change the font size, \(AppShortcut.resetZoom.symbol) to reset it.")
            }
        }
        .formStyle(.grouped)
    }
}
