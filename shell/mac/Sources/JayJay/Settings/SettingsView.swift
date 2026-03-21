import SwiftUI

struct SettingsView: View {
    @Environment(AppSettings.self) private var settings
    @Environment(\.colorScheme) private var colorScheme
    @State private var selectedTab = 0

    var body: some View {
        ZStack {
            windowChromeBackground

            TabView(selection: $selectedTab) {
                // Appearance tab
                ScrollView {
                    VStack(alignment: .leading, spacing: 20) {
                        SettingsSectionCard {
                            VStack(alignment: .leading, spacing: 18) {
                                VStack(alignment: .leading, spacing: 10) {
                                    settingsLabel("Theme", description: "Follow macOS or lock JayJay to a preferred look.")
                                    Picker("Theme", selection: appearanceBinding) {
                                        ForEach(AppSettings.AppearanceMode.allCases) { mode in
                                            Text(mode.title).tag(mode)
                                        }
                                    }
                                    .labelsHidden()
                                    .pickerStyle(.segmented)
                                }

                                Divider()

                                VStack(alignment: .leading, spacing: 10) {
                                    settingsLabel("Font Size", description: "Scale the interface for readability.")
                                    HStack(spacing: 14) {
                                        Slider(value: fontScaleBinding, in: 0.85...1.45, step: 0.05)
                                            .tint(Color(red: 0.18, green: 0.41, blue: 0.9))
                                        Text(fontScaleLabel)
                                            .jayjayFont(12, weight: .semibold, design: .monospaced)
                                            .foregroundStyle(Color.primary.opacity(0.7))
                                            .padding(.horizontal, 10).padding(.vertical, 6)
                                            .background(Capsule().fill(cardInsetFill))
                                    }
                                }
                            }
                        }

                        HStack {
                            Button("Reset Defaults") {
                                settings.fontScale = 1.0
                                settings.appearanceMode = .system
                            }
                            .buttonStyle(.bordered)
                            Spacer()
                        }
                    }
                    .padding(22)
                }
                .tag(0)
                .tabItem { Label("Appearance", systemImage: "paintbrush") }

                // Diff tab
                ScrollView {
                    VStack(alignment: .leading, spacing: 20) {
                        SettingsSectionCard {
                            VStack(spacing: 14) {
                                SettingsToggleRow(title: "Side-by-side diff",
                                    description: "Show before and after in two synchronized columns.",
                                    isOn: sideBySideBinding)
                                SettingsToggleRow(title: "Ignore whitespace changes",
                                    description: "Reduce noise from formatting-only edits.",
                                    isOn: ignoreWhitespaceBinding)
                                SettingsToggleRow(title: "Tree view for files",
                                    description: "Group changed files by folders.",
                                    isOn: treeFileListBinding)
                                SettingsToggleRow(title: "Skip abandon confirmation",
                                    description: "Don't ask before abandoning changes.",
                                    isOn: skipAbandonBinding)
                            }
                        }

                        HStack {
                            Button("Reset Defaults") {
                                settings.sideBySideDiff = false
                                settings.ignoreWhitespace = false
                                settings.treeFileList = false
                            }
                            .buttonStyle(.bordered)
                            Spacer()
                        }
                    }
                    .padding(22)
                }
                .tag(1)
                .tabItem { Label("Diff", systemImage: "doc.text.magnifyingglass") }

                // Tools tab
                ScrollView {
                    VStack(alignment: .leading, spacing: 20) {
                        SettingsSectionCard {
                            HStack {
                                Text("Editor").jayjayFont(13, weight: .medium)
                                Spacer()
                                Picker("", selection: editorBinding) {
                                    ForEach(AppSettings.ExternalEditor.allCases) { editor in
                                        Text(editor.title).tag(editor)
                                    }
                                }
                                .labelsHidden()
                                .pickerStyle(.menu)
                                .fixedSize()
                            }

                            if settings.externalEditor == .custom {
                                TextField("Command (e.g. code, nvim)", text: customEditorBinding)
                                    .textFieldStyle(.roundedBorder)
                                    .jayjayFont(12, design: .monospaced)
                            }

                            Divider()

                            HStack {
                                Text("Terminal").jayjayFont(13, weight: .medium)
                                Spacer()
                                Picker("", selection: terminalBinding) {
                                    ForEach(AppSettings.Terminal.allCases) { term in
                                        Text(term.title).tag(term)
                                    }
                                }
                                .labelsHidden()
                                .pickerStyle(.menu)
                                .fixedSize()
                            }

                            if settings.terminal == .custom {
                                TextField("App name (e.g. Terminal)", text: customTerminalBinding)
                                    .textFieldStyle(.roundedBorder)
                                    .jayjayFont(12, design: .monospaced)
                            }
                        }

                        HStack {
                            Button("Reset Defaults") {
                                settings.externalEditor = .vscode
                                settings.customEditorCommand = ""
                                settings.terminal = .terminal
                                settings.customTerminalCommand = ""
                            }
                            .buttonStyle(.bordered)
                            Spacer()
                        }
                    }
                    .padding(22)
                }
                .tag(2)
                .tabItem { Label("Tools", systemImage: "wrench.and.screwdriver") }

                // Jujutsu tab
                ScrollView {
                    VStack(alignment: .leading, spacing: 20) {
                        SettingsSectionCard {
                            JJConfigView()
                        }
                    }
                    .padding(22)
                }
                .tag(3)
                .tabItem { Label("Jujutsu", systemImage: "arrow.triangle.branch") }

                AboutView(embedded: true)
                    .tag(4)
                    .tabItem { Label("About", systemImage: "info.circle") }
            }
            .padding(16)
        }
        .frame(width: 520, height: 460)
    }

    private var appearanceBinding: Binding<AppSettings.AppearanceMode> {
        Binding(
            get: { settings.appearanceMode },
            set: { settings.appearanceMode = $0 }
        )
    }

    private var fontScaleBinding: Binding<Double> {
        Binding(
            get: { settings.fontScale },
            set: { settings.fontScale = $0 }
        )
    }

    private var sideBySideBinding: Binding<Bool> {
        Binding(
            get: { settings.sideBySideDiff },
            set: { settings.sideBySideDiff = $0 }
        )
    }

    private var ignoreWhitespaceBinding: Binding<Bool> {
        Binding(
            get: { settings.ignoreWhitespace },
            set: { settings.ignoreWhitespace = $0 }
        )
    }

    private var treeFileListBinding: Binding<Bool> {
        Binding(
            get: { settings.treeFileList },
            set: { settings.treeFileList = $0 }
        )
    }

    private var skipAbandonBinding: Binding<Bool> {
        Binding(
            get: { settings.skipAbandonConfirmation },
            set: { settings.skipAbandonConfirmation = $0 }
        )
    }

    private var editorBinding: Binding<AppSettings.ExternalEditor> {
        Binding(
            get: { settings.externalEditor },
            set: { settings.externalEditor = $0 }
        )
    }

    private var customEditorBinding: Binding<String> {
        Binding(
            get: { settings.customEditorCommand },
            set: { settings.customEditorCommand = $0 }
        )
    }

    private var terminalBinding: Binding<AppSettings.Terminal> {
        Binding(
            get: { settings.terminal },
            set: { settings.terminal = $0 }
        )
    }

    private var customTerminalBinding: Binding<String> {
        Binding(
            get: { settings.customTerminalCommand },
            set: { settings.customTerminalCommand = $0 }
        )
    }

    private var fontScaleLabel: String {
        "\(Int(settings.fontScale * 100))%"
    }

    private func settingsLabel(_ title: String, description: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .jayjayFont(13, weight: .semibold)
                .foregroundStyle(Color.primary.opacity(0.8))
            Text(description)
                .jayjayFont(11)
                .foregroundStyle(Color.secondary)
                .fixedSize(horizontal: false, vertical: true)
        }
    }

    private var cardInsetFill: Color {
        colorScheme == .dark ? Color.white.opacity(0.12) : Color.white.opacity(0.86)
    }

    private var cardStroke: Color {
        colorScheme == .dark ? Color.white.opacity(0.16) : Color.white.opacity(0.82)
    }

    private var windowChromeBackground: some View {
        LinearGradient(
            colors: colorScheme == .dark
                ? [Color(red: 0.12, green: 0.16, blue: 0.28), Color(red: 0.14, green: 0.22, blue: 0.4), Color(red: 0.11, green: 0.15, blue: 0.26)]
                : [Color(red: 0.95, green: 0.98, blue: 1.0), Color(red: 0.84, green: 0.91, blue: 1.0), Color(red: 0.73, green: 0.82, blue: 0.99)],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        .ignoresSafeArea()
    }

    private var contentChromeFill: Color {
        colorScheme == .dark
            ? Color(red: 0.11, green: 0.13, blue: 0.19).opacity(0.92)
            : Color.white.opacity(0.82)
    }

    private var contentChromeStroke: Color {
        colorScheme == .dark
            ? Color.white.opacity(0.08)
            : Color.white.opacity(0.92)
    }

    private var contentChromeShadow: Color {
        colorScheme == .dark
            ? Color.black.opacity(0.34)
            : Color(red: 0.2, green: 0.3, blue: 0.54).opacity(0.12)
    }
}
