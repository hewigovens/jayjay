import AppKit
import SwiftUI

struct RepoTitleMenu: View {
    let repoPath: String

    @Environment(RepositoryStore.self) private var repositoryStore
    @Environment(RepoWindowManager.self) private var windowManager

    private var standardizedRepoPath: String {
        URL(fileURLWithPath: repoPath).standardizedFileURL.path
    }

    private var closedPinnedRepositories: [String] {
        let open = Set(windowManager.openRepoPaths)
        return repositoryStore.paths.filter { !open.contains($0) }
    }

    var body: some View {
        Menu {
            Section("Open Windows") {
                ForEach(windowManager.openRepoPaths, id: \.self) { path in
                    Button {
                        afterMenuDismiss { windowManager.activateRepo(path) }
                    } label: {
                        Label(
                            URL(fileURLWithPath: path).repositoryDisplayName,
                            systemImage: path == standardizedRepoPath ? "checkmark" : "macwindow"
                        )
                    }
                }
            }

            if !closedPinnedRepositories.isEmpty {
                Section("Pinned") {
                    ForEach(closedPinnedRepositories, id: \.self) { path in
                        Button {
                            afterMenuDismiss { windowManager.openRepo(path) }
                        } label: {
                            Label(
                                URL(fileURLWithPath: path).repositoryDisplayName,
                                systemImage: "pin.fill"
                            )
                        }
                    }
                }
            }

            Divider()

            Button {
                afterMenuDismiss { windowManager.showRepoList() }
            } label: {
                Label("Repository List...", systemImage: "list.bullet")
            }

            Button {
                afterMenuDismiss { windowManager.openRepositoryPicker() }
            } label: {
                Label("Open Repository...", systemImage: "folder")
            }
        } label: {
            HStack(spacing: 4) {
                Text(URL(fileURLWithPath: repoPath).repositoryDisplayName)
                    .fontWeight(.semibold)
                Image(systemName: "chevron.down")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 8)
            .frame(minHeight: 30)
            .contentShape(Rectangle())
        }
        .menuIndicator(.hidden)
        .menuStyle(.button)
        .buttonStyle(.plain)
        .fixedSize()
        .onAppear(perform: refresh)
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in
            refresh()
        }
        .accessibilityLabel("Switch Repository")
    }

    private func refresh() {
        repositoryStore.reload()
        windowManager.refreshOpenRepoPaths()
    }

    private func afterMenuDismiss(_ action: @escaping @MainActor () -> Void) {
        // Window operations can be dropped while AppKit is still tracking the menu, so dispatch them on the next main-queue turn.
        DispatchQueue.main.async(execute: action)
    }
}
