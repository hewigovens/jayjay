import JayJayCore
import SwiftUI

struct KeyboardShortcutsView: View {
    @Environment(\.dismiss) private var dismiss
    @AppStorage(AppSettings.settingsPageKey) private var settingsPage = SettingsPage.appearance
    @Environment(\.openSettings) private var openSettings
    @Environment(ShortcutStore.self) private var store

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            ScrollView {
                HStack(alignment: .top, spacing: 32) {
                    ForEach(Array(ShortcutGuide.columns(bindings: store.bindings).enumerated()), id: \.offset) { _, column in
                        VStack(alignment: .leading, spacing: 20) {
                            ForEach(column) { section in
                                sectionBlock(section)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .topLeading)
                    }
                }
                .padding(20)
            }
            Divider()
            footer
        }
        .frame(width: 720, height: 560)
        .onExitCommand { dismiss() }
    }

    private var header: some View {
        ZStack {
            HStack(spacing: 8) {
                Image(systemName: "keyboard")
                    .font(.system(size: 15))
                    .foregroundStyle(.secondary)
                Text(Localized.Menu.keyboardShortcuts)
                    .jayjayFont(16, weight: .semibold)
            }
            HStack {
                Spacer()
                Button { dismiss() } label: {
                    Image(systemName: "xmark")
                        .font(.system(size: 11, weight: .bold))
                        .foregroundStyle(.secondary)
                        .padding(6)
                        .background(Color.primary.opacity(0.06), in: Circle())
                }
                .buttonStyle(.plain)
                .help(Localized.Common.close)
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 14)
    }

    private func sectionBlock(_ section: ShortcutSection) -> some View {
        VStack(alignment: .leading, spacing: 9) {
            Text(section.title)
                .jayjayFont(12, weight: .bold)
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
            ForEach(section.entries) { entry in
                HStack(spacing: 12) {
                    Text(entry.title)
                        .jayjayFont(13)
                        .lineLimit(1)
                    Spacer(minLength: 8)
                    ShortcutKeyCaps(chords: entry.chords)
                }
            }
        }
    }

    private var footer: some View {
        HStack(spacing: 0) {
            Text(Localized.Shortcuts.escClosesThePaletteAndSheetsN)
                .jayjayFont(11)
                .foregroundStyle(.tertiary)
            Spacer()
            Button(Localized.Shortcuts.customize) {
                settingsPage = .shortcuts
                openSettings()
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 10)
    }
}
