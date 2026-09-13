import AppKit
import SwiftUI

struct JJConfigView: View {
    @Environment(AppSettings.self) private var settings
    @State private var config = SettingsSnapshot<JjConfigSnapshot>()

    var body: some View {
        Group {
            if let snapshot = config.value {
                switch snapshot {
                    case .notInstalled:
                        emptyState("jj is not installed")
                    case .missing:
                        emptyState("Config not found")
                    case let .found(path, sections):
                        Section {
                            configPathRow(path: path)
                        }
                        ForEach(sections) { section in
                            Section(section.name) {
                                ForEach(section.entries) { entry in
                                    configRow(key: entry.key, value: entry.value, icon: entry.icon)
                                }
                            }
                            .id(section.id)
                        }
                }
            } else {
                ProgressView()
                    .controlSize(.small)
                    .frame(maxWidth: .infinity, minHeight: 80)
            }
        }
        .task { await config.load { JjConfigSnapshot.load() } }
    }

    private func emptyState(_ message: String) -> some View {
        Section {
            Text(message)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, minHeight: 80, alignment: .center)
                .accessibilityIdentifier(AID.Settings.jjConfigMissing)
        }
    }

    private func configPathRow(path: String) -> some View {
        HStack {
            Text(path)
                .font(.system(size: 11, design: .monospaced))
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
            Spacer()
            Button("Open") {
                if !settings.openInEditor(absolutePath: path) {
                    NSWorkspace.shared.open(URL(fileURLWithPath: path))
                }
            }
            .controlSize(.small)
        }
    }

    private func configRow(key: String, value: String, icon: String) -> some View {
        HStack {
            HStack(spacing: 6) {
                Image(systemName: icon)
                    .frame(width: 16, alignment: .center)
                    .foregroundStyle(.secondary)
                Text(key)
            }
            Spacer()
            Text(value)
                .font(.system(size: 12, design: .monospaced))
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
                .lineLimit(1)
                .truncationMode(.middle)
        }
    }
}
