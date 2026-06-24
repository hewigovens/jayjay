import SwiftUI

/// A 1Password-style keyboard cheatsheet shown on ⌘/. Two balanced columns of sectioned shortcuts; closes with Esc, the X, or the window's close button.
struct KeyboardShortcutsView: View {
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            ScrollView {
                HStack(alignment: .top, spacing: 32) {
                    ForEach(Array(ShortcutGuide.columns.enumerated()), id: \.offset) { _, column in
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
                Text("Keyboard Shortcuts")
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
                .help("Close")
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
                    Text(entry.label)
                        .jayjayFont(13)
                        .lineLimit(1)
                    Spacer(minLength: 8)
                    KeyCapRow(keys: entry.keys)
                }
            }
        }
    }

    private var footer: some View {
        HStack(spacing: 0) {
            Text("Esc closes the palette and sheets · ⌃N / ⌃P also move the selection")
                .jayjayFont(11)
                .foregroundStyle(.tertiary)
            Spacer()
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 10)
    }
}

/// Renders a shortcut as discrete key-cap chips.
private struct KeyCapRow: View {
    let keys: [String]

    var body: some View {
        HStack(spacing: 4) {
            ForEach(Array(keys.enumerated()), id: \.offset) { _, key in
                Text(key)
                    .font(.system(size: 12, weight: .medium, design: .rounded))
                    .foregroundStyle(.primary)
                    .frame(minWidth: 18)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                    .background(Color.primary.opacity(0.07), in: RoundedRectangle(cornerRadius: 5))
                    .overlay(
                        RoundedRectangle(cornerRadius: 5)
                            .stroke(Color.primary.opacity(0.12), lineWidth: 0.5)
                    )
            }
        }
    }
}
