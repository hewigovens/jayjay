import SwiftUI

struct WelcomeView: View {
    static let minimumSize = NSSize(width: 480, height: 600)

    let onOpen: (String) -> Void

    @Environment(AppSettings.self) private var settings
    @Environment(RepositoryStore.self) private var repositoryStore
    /// Cached per-path lookups, resolved off the main actor so entries on a slow volume render flat instead of stalling the list. Re-resolved on app activation: a path can be replaced externally while the list stays open.
    @State private var resolutions: [String: RepoPathResolution] = [:]
    @State private var resolutionEpoch = 0

    private struct ResolutionRequest: Equatable {
        let epoch: Int
        let paths: [String]
    }

    var body: some View {
        let pinnedRepositories = repositoryStore.paths
        let pinned = Set(pinnedRepositories)
        let recentRepositories = settings.recentRepos.filter { !pinned.contains($0) }
        let hasRepositories = !pinnedRepositories.isEmpty || !recentRepositories.isEmpty

        VStack(spacing: 0) {
            if !hasRepositories {
                Spacer()
                header.padding(.horizontal, 30)
                Spacer()
            } else {
                header
                    .padding(.top, 30)
                    .padding(.bottom, 22)
                    .padding(.horizontal, 30)
                Divider()
                repositorySections(
                    pinnedRepositories: pinnedRepositories,
                    recentRepositories: recentRepositories
                )
            }
        }
        .frame(
            minWidth: Self.minimumSize.width,
            maxWidth: .infinity,
            minHeight: Self.minimumSize.height,
            maxHeight: .infinity
        )
        .onAppear { repositoryStore.reload() }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in
            repositoryStore.reload()
            resolutionEpoch += 1
        }
        .task(id: ResolutionRequest(epoch: resolutionEpoch, paths: pinnedRepositories + recentRepositories)) {
            await resolveRepoPaths(pinnedRepositories + recentRepositories)
        }
    }

    private func resolveRepoPaths(_ paths: [String]) async {
        guard !paths.isEmpty else { return }
        let resolved = await RepoListGrouping.resolve(paths: paths)
        guard !Task.isCancelled, resolved != resolutions else { return }
        resolutions = resolved
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

    private func repositorySections(
        pinnedRepositories: [String],
        recentRepositories: [String]
    ) -> some View {
        let groups = RepoListGrouping.groups(
            pinned: pinnedRepositories,
            recents: recentRepositories,
            resolutions: resolutions
        )
        return ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                if !groups.pinned.isEmpty {
                    repositorySection(title: "Pinned") {
                        ForEach(groups.pinned) { group in
                            repoGroupRows(group, pinned: true)
                        }
                    }
                }

                if !groups.recent.isEmpty {
                    repositorySection(title: "Recent Repositories", showsClear: true) {
                        ForEach(groups.recent) { group in
                            repoGroupRows(group, pinned: false)
                        }
                    }
                }
            }
            .padding(.horizontal, 30)
            .padding(.vertical, 18)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
    }

    @ViewBuilder
    private func repoGroupRows(_ group: RepoGroup, pinned: Bool) -> some View {
        if group.workspaces.isEmpty {
            repoRow(path: group.path, pinned: pinned)
        } else {
            groupedRepoCard(group, pinned: pinned)
        }
    }

    /// One card per repo with workspaces: the repo name as header, then the default workspace and every listed sibling as open targets.
    private func groupedRepoCard(_ group: RepoGroup, pinned: Bool) -> some View {
        VStack(alignment: .leading, spacing: 7) {
            HStack(spacing: 8) {
                Text(URL(fileURLWithPath: group.path).repositoryDisplayName)
                    .jayjayFont(12, weight: .semibold)
                Spacer()
                pinAndRemoveButtons(path: group.path, pinned: pinned)
            }
            workspaceEntryRow(name: "default", path: group.path, nested: false)
            ForEach(group.workspaces, id: \.self) { path in
                workspaceEntryRow(name: URL(fileURLWithPath: path).lastPathComponent, path: path, nested: true)
            }
        }
        .padding(8)
        .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func repositorySection(
        title: String,
        showsClear: Bool = false,
        @ViewBuilder content: () -> some View
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text(title)
                    .jayjayFont(13, weight: .semibold)
                Spacer()
                if showsClear {
                    Button("Clear") {
                        settings.recentRepos = []
                        settings.lastOpenedRepo = nil
                    }
                    .controlSize(.small)
                }
            }
            VStack(alignment: .leading, spacing: 10, content: content)
        }
    }

    private func repoRow(path: String, pinned: Bool) -> some View {
        HStack(spacing: 8) {
            Button { onOpen(path) } label: {
                VStack(alignment: .leading, spacing: 2) {
                    Text(URL(fileURLWithPath: path).repositoryDisplayName)
                        .jayjayFont(12, weight: .medium)
                    Text(path)
                        .jayjayFont(10)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            pinAndRemoveButtons(path: path, pinned: pinned)
        }
        .padding(8)
        .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    @ViewBuilder
    private func pinAndRemoveButtons(path: String, pinned: Bool) -> some View {
        Button { repositoryStore.setPinned(!pinned, path: path) } label: {
            Image(systemName: pinned ? "pin.slash.fill" : "pin.fill")
                .foregroundStyle(.tertiary)
        }
        .buttonStyle(.plain)
        .help(pinned ? "Unpin Repository" : "Pin Repository")

        if !pinned {
            removeRecentButton(path: path)
        }
    }

    private func removeRecentButton(path: String) -> some View {
        Button { settings.removeRecentRepo(path) } label: {
            Image(systemName: "xmark.circle.fill")
                .foregroundStyle(.tertiary)
        }
        .buttonStyle(.plain)
        .help("Remove from Recent")
    }

    /// The card's own row (`nested: false`) is the primary repo, whose pin and remove live in the card header; nested rows are always unpinned recents, so pinning one promotes it top-level.
    private func workspaceEntryRow(name: String, path: String, nested: Bool) -> some View {
        HStack(spacing: 8) {
            Button { onOpen(path) } label: {
                HStack(spacing: 6) {
                    Image(systemName: "folder")
                        .jayjayFont(10)
                        .foregroundStyle(.secondary)
                    Text(name)
                        .jayjayFont(12, weight: .medium)
                    Text(path)
                        .jayjayFont(10)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            if nested {
                pinAndRemoveButtons(path: path, pinned: false)
            }
        }
        .padding(.leading, 6)
    }
}
