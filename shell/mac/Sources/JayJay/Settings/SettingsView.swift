import JayJayCore
import SwiftUI

struct SettingsView: View {
    @ObservedObject var updater: SparkleUpdater
    let windowManager: RepoWindowManager
    @State private var selection = SettingsPage.appearance
    @State private var toolAvailability = SettingsSnapshot<[String: Bool]>()
    @State private var cliDiagnostics = SettingsSnapshot<[String: CliStatus]>()
    @State private var jjConfig = SettingsSnapshot<JjConfigSnapshot>()

    var body: some View {
        NavigationSplitView(columnVisibility: .constant(.all)) {
            List(SettingsPage.allCases, selection: $selection) { page in
                Label {
                    Text(page.title)
                } icon: {
                    Image(systemName: page.symbol)
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(.white)
                        .frame(width: 20, height: 20)
                        .background(page.color.gradient, in: RoundedRectangle(cornerRadius: 5))
                }
                .padding(.vertical, 4)
                .tag(page)
                .accessibilityIdentifier(AID.Settings.page(page.rawValue))
            }
            .listStyle(.sidebar)
            .accessibilityIdentifier(AID.Settings.sidebar)
            .navigationSplitViewColumnWidth(190)
            .toolbar(removing: .sidebarToggle)
        } detail: {
            detail
                .navigationTitle(selection.title)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .frame(minWidth: 720, idealWidth: 760, minHeight: 500, idealHeight: 560)
        .onExitCommand { NSApp.keyWindow?.close() }
    }

    @ViewBuilder
    private var detail: some View {
        switch selection {
            case .appearance:
                SettingsAppearancePage()
            case .diff:
                SettingsDiffPage()
            case .workflow:
                SettingsWorkflowPage()
            case .integrations:
                Form {
                    SettingsToolSections(availability: toolAvailability)
                    SettingsCLISections(diagnostics: cliDiagnostics)
                }
                .formStyle(.grouped)
            case .jujutsu:
                Form {
                    JJConfigView(config: jjConfig)
                }
                .formStyle(.grouped)
            case .dataPrivacy:
                SettingsDataPrivacyPage(windowManager: windowManager)
            case .about:
                ScrollView {
                    AboutView(embedded: true, updater: updater)
                }
        }
    }
}
