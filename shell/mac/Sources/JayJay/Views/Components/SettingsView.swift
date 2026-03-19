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
                        SettingsSectionCard(
                            title: "Appearance",
                            subtitle: "Keep long review sessions comfortable without losing density."
                        ) {
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
                        SettingsSectionCard(
                            title: "Diff",
                            subtitle: "Tune how history and file changes are presented."
                        ) {
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

                // Jujutsu tab
                ScrollView {
                    VStack(alignment: .leading, spacing: 20) {
                        SettingsSectionCard(
                            title: "Jujutsu Configuration",
                            subtitle: "Read-only view of your jj config."
                        ) {
                            JJConfigView()
                        }
                    }
                    .padding(22)
                }
                .tag(2)
                .tabItem { Label("Jujutsu", systemImage: "arrow.triangle.branch") }

                AboutView(embedded: true)
                    .tag(3)
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
        ZStack {
            LinearGradient(
                colors: colorScheme == .dark
                    ? [
                        Color(red: 0.08, green: 0.1, blue: 0.16),
                        Color(red: 0.1, green: 0.13, blue: 0.22),
                        Color(red: 0.08, green: 0.09, blue: 0.14)
                    ]
                    : [
                        Color(red: 0.97, green: 0.98, blue: 1.0),
                        Color(red: 0.93, green: 0.96, blue: 1.0),
                        Color(red: 0.95, green: 0.95, blue: 0.98)
                    ],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )

            Circle()
                .fill(
                    LinearGradient(
                        colors: [
                            Color(red: 0.22, green: 0.47, blue: 0.98).opacity(colorScheme == .dark ? 0.18 : 0.16),
                            Color(red: 0.53, green: 0.77, blue: 1.0).opacity(0.03)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .frame(width: 260, height: 260)
                .blur(radius: 14)
                .offset(x: 180, y: -180)

            Circle()
                .fill(
                    LinearGradient(
                        colors: [
                            Color(red: 0.09, green: 0.29, blue: 0.76).opacity(colorScheme == .dark ? 0.2 : 0.12),
                            Color(red: 0.2, green: 0.48, blue: 0.95).opacity(0.02)
                        ],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                )
                .frame(width: 220, height: 220)
                .blur(radius: 18)
                .offset(x: -200, y: 190)
        }
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

private struct SettingsSectionCard<Content: View>: View {
    let title: String
    let subtitle: String
    let content: Content

    @Environment(\.colorScheme) private var colorScheme

    init(
        title: String,
        subtitle: String,
        @ViewBuilder content: () -> Content
    ) {
        self.title = title
        self.subtitle = subtitle
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .jayjayFont(16, weight: .semibold)
                    .foregroundStyle(Color.primary.opacity(0.84))
                Text(subtitle)
                    .jayjayFont(12)
                    .foregroundStyle(Color.secondary)
            }

            content
        }
        .padding(20)
        .background(
            RoundedRectangle(cornerRadius: 24, style: .continuous)
                .fill(cardFill)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 24, style: .continuous)
                .stroke(cardStroke, lineWidth: 1)
        )
    }

    private var cardFill: Color {
        colorScheme == .dark ? Color.white.opacity(0.08) : Color.white.opacity(0.72)
    }

    private var cardStroke: Color {
        colorScheme == .dark ? Color.white.opacity(0.14) : Color.white.opacity(0.82)
    }
}

private struct SettingsToggleRow: View {
    let title: String
    let description: String
    @Binding var isOn: Bool
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .jayjayFont(13, weight: .semibold)
                    .foregroundStyle(Color.primary.opacity(0.8))
                Text(description)
                    .jayjayFont(11)
                    .foregroundStyle(Color.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Spacer(minLength: 16)

            Toggle("", isOn: $isOn)
                .labelsHidden()
                .toggleStyle(.switch)
                .tint(Color(red: 0.18, green: 0.41, blue: 0.9))
        }
        .padding(14)
        .background(
            RoundedRectangle(cornerRadius: 18, style: .continuous)
                .fill(colorScheme == .dark ? Color.white.opacity(0.06) : Color.white.opacity(0.54))
        )
    }
}

private struct JJConfigView: View {
    @State private var configText: String?
    @State private var configPath = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            if !configPath.isEmpty {
                HStack {
                    Text(configPath)
                        .jayjayFont(11, design: .monospaced)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                    Spacer()
                    Button("Open") {
                        NSWorkspace.shared.open(URL(fileURLWithPath: configPath))
                    }
                    .controlSize(.small)
                }
            }

            Group {
                if let config = configText {
                    Text(config)
                        .jayjayFont(12, design: .monospaced)
                        .textSelection(.enabled)
                } else {
                    ProgressView()
                        .controlSize(.small)
                }
            }
            .padding(10)
            .frame(maxWidth: .infinity, minHeight: 80, alignment: .topLeading)
            .background(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(Color.primary.opacity(0.04))
            )
        }
        .task {
            let config = Self.runShell("jj config list")
            let path = Self.runShell("jj config path --user")
            configText = config
            configPath = path
        }
    }

    private static func runShell(_ command: String) -> String {
        let proc = Process()
        let pipe = Pipe()
        proc.standardOutput = pipe
        proc.standardError = pipe
        proc.executableURL = URL(fileURLWithPath: "/bin/bash")
        proc.arguments = ["-c", command]
        try? proc.run()
        proc.waitUntilExit()
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        return (String(data: data, encoding: .utf8) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }

}
