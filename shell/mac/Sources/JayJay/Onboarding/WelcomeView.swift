import SwiftUI

struct WelcomeView: View {
    let onOpen: (String) -> Void

    @Environment(AppSettings.self) private var settings

    var body: some View {
        VStack(spacing: 22) {
            VStack(spacing: 12) {
                Image(nsImage: NSApplication.shared.applicationIconImage)
                    .resizable()
                    .frame(width: 80, height: 80)
                Text("JayJay")
                    .jayjayFont(28, weight: .bold)
                Text("A native GUI for Jujutsu")
                    .jayjayFont(14)
                    .foregroundStyle(.secondary)
                Button("Open Repository...") {
                    let panel = NSOpenPanel()
                    panel.canChooseFiles = false
                    panel.canChooseDirectories = true
                    panel.allowsMultipleSelection = false
                    if panel.runModal() == .OK, let url = panel.url {
                        onOpen(url.path)
                    }
                }
                .keyboardShortcut(.defaultAction)
            }

            if !settings.recentRepos.isEmpty {
                recentReposList
            }
        }
        .frame(maxWidth: 400)
        .padding(30)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var recentReposList: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text("Recent Repositories")
                    .jayjayFont(13, weight: .semibold)
                Spacer()
                Button("Clear") {
                    settings.recentRepos = []
                    settings.lastOpenedRepo = nil
                }
                .controlSize(.small)
            }

            ForEach(settings.recentRepos, id: \.self) { path in
                HStack(spacing: 8) {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(URL(fileURLWithPath: path).lastPathComponent)
                            .jayjayFont(12, weight: .medium)
                        Text(path)
                            .jayjayFont(10)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }
                    Spacer(minLength: 0)
                    Button { settings.removeRecentRepo(path) } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.tertiary)
                    }
                    .buttonStyle(.plain)
                }
                .padding(8)
                .contentShape(Rectangle())
                .onTapGesture { onOpen(path) }
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
            }
        }
    }
}
