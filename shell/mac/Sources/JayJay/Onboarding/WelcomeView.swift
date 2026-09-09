import JayJayCore
import SwiftUI

struct WelcomeView: View {
    static let minimumSize = NSSize(width: 480, height: 600)
    static let defaultSize = NSSize(width: 760, height: 600)

    let onOpen: (String) -> Void

    @Environment(AppSettings.self) private var settings
    @Environment(RepositoryStore.self) private var repositoryStore
    @State private var model = RepoListViewModel()

    var body: some View {
        let pinnedRepositories = repositoryStore.paths
        let recentRepositories = settings.recentRepos

        NavigationSplitView(columnVisibility: recentPanelVisibility) {
            recentPanel(model.groups.recent)
                .navigationSplitViewColumnWidth(280)
        } detail: {
            detail(pinned: model.groups.pinned)
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
            model.regroup()
        }
        // Keyed on both lists: moving a path between them leaves their concatenation unchanged.
        .onChange(of: [pinnedRepositories, recentRepositories], initial: true) {
            model.show(pinned: pinnedRepositories, recents: recentRepositories)
        }
    }

    /// The split view reports .automatic and .doubleColumn as well; anything but hidden counts as shown.
    private var recentPanelVisibility: Binding<NavigationSplitViewVisibility> {
        Binding(
            get: { settings.showsRecentRepositoriesPanel ? .all : .detailOnly },
            set: { settings.showsRecentRepositoriesPanel = $0 != .detailOnly }
        )
    }

    @ViewBuilder
    private func recentPanel(_ recent: [RepoGroup]) -> some View {
        if recent.isEmpty {
            Text("No Recent Repositories")
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            ScrollView {
                repositorySection(title: "Recent Repositories", showsClear: true) {
                    ForEach(recent) { group in
                        repoGroupRows(group, pinned: false)
                    }
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 18)
            }
        }
    }

    /// The detail keeps the single-column welcome width so a hidden panel widens the margins, not the cards.
    private func detail(pinned: [RepoGroup]) -> some View {
        Group {
            if pinned.isEmpty {
                VStack(spacing: 0) {
                    Spacer()
                    header.padding(.horizontal, 30)
                    Spacer()
                }
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 0) {
                        header
                            .padding(.top, 30)
                            .padding(.bottom, 22)
                            .padding(.horizontal, 30)
                        Divider()
                        repositorySection(title: "Pinned") {
                            ForEach(pinned) { group in
                                repoGroupRows(group, pinned: true)
                            }
                        }
                        .padding(.horizontal, 30)
                        .padding(.vertical, 18)
                    }
                    .frame(maxWidth: Self.minimumSize.width)
                    .frame(maxWidth: .infinity)
                }
            }
        }
        .frame(minWidth: Self.minimumSize.width, maxWidth: .infinity, maxHeight: .infinity)
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

    @ViewBuilder
    private func repoGroupRows(_ group: RepoGroup, pinned: Bool) -> some View {
        if group.workspaces.isEmpty {
            repoRow(path: group.path, pinned: pinned)
        } else {
            groupedRepoCard(group, pinned: pinned)
        }
    }

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
        Button { repositoryStore.setPinned(!pinned, path: path, keepingListedIn: settings) } label: {
            Image(systemName: pinned ? "pin.slash.fill" : "pin.fill")
                .foregroundStyle(.tertiary)
        }
        .buttonStyle(.plain)
        .help(pinned ? "Unpin Repository" : "Pin Repository")

        if !pinned {
            Button { settings.removeRecentRepo(path) } label: {
                Image(systemName: "xmark.circle.fill")
                    .foregroundStyle(.tertiary)
            }
            .buttonStyle(.plain)
            .help("Remove from Recent")
        }
    }

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
                pinAndRemoveButtons(path: path, pinned: repositoryStore.paths.contains(path))
            }
        }
        .padding(.leading, 6)
    }
}

extension RepoGroup: @retroactive Identifiable {
    public var id: String {
        path
    }
}
