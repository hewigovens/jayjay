import JayJayCore
import SwiftUI

struct SettingsToolSections: View {
    @Environment(AppSettings.self) private var settings
    let availability: SettingsSnapshot<[AiProvider: Bool]>

    var body: some View {
        Group {
            Section(Localized.Settings.applications) {
                Picker(selection: Binding(
                    get: { settings.externalEditor },
                    set: { settings.externalEditor = $0 }
                )) {
                    ForEach(AppSettings.ExternalEditor.allCases) { editor in
                        Text(editor.title).tag(editor)
                    }
                } label: {
                    SettingsLabel(Localized.Settings.editor, icon: "curlybraces")
                }
                if settings.externalEditor == .custom {
                    TextField(Localized.Settings.command, text: Binding(
                        get: { settings.customEditorCommand },
                        set: { settings.customEditorCommand = $0 }
                    ), prompt: Text(Localized.Settings.example(example: "code, nvim")))
                }
                Picker(selection: Binding(
                    get: { settings.terminal },
                    set: { settings.terminal = $0 }
                )) {
                    ForEach(AppSettings.Terminal.allCases) { term in
                        Text(term.title).tag(term)
                    }
                } label: {
                    SettingsLabel(Localized.Settings.terminal, icon: "terminal")
                }
                if settings.terminal == .custom {
                    TextField(Localized.Settings.appName, text: Binding(
                        get: { settings.customTerminalCommand },
                        set: { settings.customTerminalCommand = $0 }
                    ), prompt: Text(Localized.Settings.example(example: "Terminal")))
                }
            }

            Section {
                List {
                    ForEach(settings.aiProviderOrder, id: \.self) { provider in
                        providerRow(provider)
                            .alignmentGuide(.listRowSeparatorLeading) { _ in 0 }
                            .frame(height: Self.providerRowHeight)
                            .listRowInsets(Self.providerRowInsets)
                    }
                    .onMove { settings.aiProviderOrder.move(fromOffsets: $0, toOffset: $1) }
                }
                .listStyle(.plain)
                .scrollDisabled(true)
                .scrollContentBackground(.hidden)
                .frame(height: CGFloat(settings.aiProviderOrder.count) * Self.providerRowHeight)
            } header: {
                Text(Localized.Settings.aiProviders)
            } footer: {
                Text(Localized.Settings.aiProvidersDetail)
            }
        }
        .task {
            let providers = settings.aiProviderOrder
            await availability.load {
                Dictionary(uniqueKeysWithValues: providers.map { ($0, $0.isReady) })
            }
        }
    }

    /// `onMove` needs a `List`, which does not self-size in a `Form`; these match the grouped form rows around it.
    private static let providerRowHeight: CGFloat = 37
    private static let providerRowInsets = EdgeInsets(top: 0, leading: 4, bottom: 0, trailing: 4)

    private func providerRow(_ provider: AiProvider) -> some View {
        HStack {
            SettingsLabel(provider.label, icon: provider.icon, iconScale: provider.iconScale)
            Spacer()
            Text(status(of: provider))
                .foregroundStyle(.secondary)
                .font(.system(size: 11))
            if let ready = availability.value?[provider] {
                Image(systemName: ready ? "checkmark.circle.fill" : "xmark.circle")
                    .foregroundStyle(ready ? Color.green : Color.secondary)
            }
        }
        .contentShape(Rectangle())
    }

    private func status(of provider: AiProvider) -> String {
        guard let ready = availability.value?[provider] else { return Localized.Common.checking }
        if provider == .appleIntelligence {
            return ready ? Localized.Settings.available : Localized.Settings.notAvailable
        }
        return ready ? Localized.Common.installed : Localized.Common.notFound
    }
}
