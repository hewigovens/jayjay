import JayJayCore
import SwiftUI

struct SettingsView: View {
    @Environment(AppSettings.self) var settings
    @ObservedObject var updater: SparkleUpdater
    let windowManager: RepoWindowManager
    @State var reviewSummary = ReviewStoreSummary(marks: 0, notes: 0)
    @State var confirmClearReviewData = false

    var body: some View {
        TabView {
            appearanceTab
                .tabItem { Label("Appearance", systemImage: "paintbrush") }
            diffTab
                .tabItem { Label("Diff", systemImage: "doc.text.magnifyingglass") }
            SettingsToolsTab()
                .tabItem { Label("Tools", systemImage: "wrench.and.screwdriver") }
            SettingsCLITab()
                .tabItem { Label("CLI", systemImage: "terminal") }
            jujutsuTab
                .tabItem { Label("Jujutsu", systemImage: "arrow.triangle.branch") }
            AboutView(embedded: true, updater: updater)
                .tabItem { Label("About", systemImage: "info.circle") }
        }
        .frame(width: 480, height: 420)
        .onExitCommand { NSApp.keyWindow?.close() }
    }

    // MARK: - Appearance

    private var appearanceTab: some View {
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
            }

            Section("Font") {
                Picker(selection: Binding(
                    get: { settings.fontFamily },
                    set: { settings.fontFamily = $0 }
                )) {
                    ForEach(AppSettings.MonoFont.allCases.filter(\.isInstalled)) { font in
                        Text(font.title).tag(font)
                    }
                } label: {
                    SettingsLabel("Family", icon: "textformat")
                }

                HStack {
                    SettingsLabel("Size", icon: "textformat.size")
                    Spacer()
                    Text("\(Int(settings.fontSize))pt")
                        .foregroundStyle(.secondary)
                        .monospacedDigit()
                    Stepper("", value: Binding(
                        get: { settings.fontSize },
                        set: { settings.fontSize = $0 }
                    ), in: 9 ... 24, step: 1)
                        .labelsHidden()
                        .controlSize(.small)
                }
            }
        }
        .formStyle(.grouped)
    }

    // MARK: - Diff

    private var diffTab: some View {
        Form {
            Section {
                Toggle(isOn: Binding(
                    get: { settings.sideBySideDiff },
                    set: { settings.sideBySideDiff = $0 }
                )) {
                    SettingsLabel("Side-by-side diff", icon: "rectangle.split.2x1")
                }
                Toggle(isOn: Binding(
                    get: { settings.ignoreWhitespace },
                    set: { settings.ignoreWhitespace = $0 }
                )) {
                    SettingsLabel("Ignore whitespace changes", icon: "space")
                }
                Toggle(isOn: Binding(
                    get: { settings.treeFileList },
                    set: { settings.treeFileList = $0 }
                )) {
                    SettingsLabel("Tree view for files", icon: "list.bullet.indent")
                }
            }

            Section("Git") {
                Toggle(isOn: Binding(
                    get: { settings.hideGitLfsDiffs },
                    set: { settings.hideGitLfsDiffs = $0 }
                )) {
                    SettingsLabel("Hide Git LFS-backed files", icon: "externaldrive")
                }
                Toggle(isOn: Binding(
                    get: { settings.enableGitSubmoduleSupport },
                    set: { settings.enableGitSubmoduleSupport = $0 }
                )) {
                    SettingsLabel("Enable Git submodule support", icon: "square.stack.3d.up")
                }
            }

            Section("Confirmations") {
                Toggle(isOn: Binding(
                    get: { settings.skipAbandonConfirmation },
                    set: { settings.skipAbandonConfirmation = $0 }
                )) {
                    SettingsLabel("Skip abandon confirmation", icon: "trash")
                }
                .accessibilityIdentifier(AID.Settings.skipAbandonConfirmation)
                Toggle(isOn: Binding(
                    get: { settings.skipWorkspaceDeleteConfirmation },
                    set: { settings.skipWorkspaceDeleteConfirmation = $0 }
                )) {
                    SettingsLabel("Skip workspace delete confirmation", icon: "folder.badge.minus")
                }
                .accessibilityIdentifier(AID.Settings.skipWorkspaceDeleteConfirmation)
                Toggle(isOn: Binding(
                    get: { settings.confirmDragRebase },
                    set: { settings.confirmDragRebase = $0 }
                )) {
                    SettingsLabel("Confirm drag-to-rebase", icon: "arrow.up.forward.app")
                }
            }

            reviewSection
        }
        .formStyle(.grouped)
    }

    // MARK: - Jujutsu

    private var jujutsuTab: some View {
        Form {
            JJConfigView()
        }
        .formStyle(.grouped)
    }
}
