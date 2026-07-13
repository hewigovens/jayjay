import SwiftUI

struct WelcomeView: View {
    static let minimumSize = NSSize(width: 480, height: 600)

    let onOpen: (String) -> Void

    @Environment(AppSettings.self) private var settings

    var body: some View {
        VStack(spacing: 0) {
            if settings.recentRepos.isEmpty {
                Spacer()
                header.padding(.horizontal, 30)
                Spacer()
            } else {
                header
                    .padding(.top, 30)
                    .padding(.bottom, 22)
                    .padding(.horizontal, 30)
                Divider()
                recentReposSection
            }
        }
        .frame(
            minWidth: Self.minimumSize.width,
            maxWidth: .infinity,
            minHeight: Self.minimumSize.height,
            maxHeight: .infinity
        )
    }

    private var header: some View {
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
        .frame(maxWidth: .infinity)
    }

    private var recentReposSection: some View {
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
            .padding(.horizontal, 30)
            .padding(.top, 18)

            ScrollView {
                VStack(alignment: .leading, spacing: 10) {
                    ForEach(settings.recentRepos, id: \.self) { path in
                        repoRow(path: path)
                    }
                }
                .padding(.horizontal, 30)
                .padding(.bottom, 18)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
    }

    private func repoRow(path: String) -> some View {
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
