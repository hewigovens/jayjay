import SwiftUI

struct WelcomeView: View {
    let onOpen: (String) -> Void

    @Environment(AppSettings.self) private var settings

    var body: some View {
        VStack(alignment: .leading, spacing: 22) {
            VStack(alignment: .leading, spacing: 12) {
                Image(systemName: "arrow.triangle.branch")
                    .font(.system(size: 48))
                    .foregroundStyle(.secondary)
                Text("JayJay")
                    .jayjayFont(34, weight: .bold)
                Text("A native GUI for Jujutsu")
                    .jayjayFont(15)
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
        .padding(24)
        .frame(minWidth: 460, minHeight: 360, alignment: .topLeading)
    }

    private var recentReposList: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text("Recent Repositories")
                    .jayjayFont(14, weight: .semibold)
                Spacer()
                Button("Clear") {
                    settings.recentRepos = []
                    settings.lastOpenedRepo = nil
                }
            }

            ForEach(settings.recentRepos, id: \.self) { path in
                HStack(spacing: 8) {
                    Button { onOpen(path) } label: {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(URL(fileURLWithPath: path).lastPathComponent)
                                .jayjayFont(13, weight: .medium)
                            Text(path)
                                .jayjayFont(11)
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                                .truncationMode(.middle)
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .buttonStyle(.plain)

                    Button { settings.removeRecentRepo(path) } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.tertiary)
                    }
                    .buttonStyle(.plain)
                }
                .padding(10)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
            }
        }
    }
}
