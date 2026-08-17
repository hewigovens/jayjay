import AppKit
import JayJayCore
import SwiftUI

/// The toolbar's combined repo and workspace switcher: a borderless "repo / workspace" button opening a filterable PickerPanel with the current repo's workspaces on top and repositories plus global actions below. Sections are rebuilt on every click, so the panel is always current.
struct RepoTitlePicker: View {
    let repoPath: String
    let workspaces: [WorkspaceInfo]
    let onOpenWorkspace: (WorkspaceInfo) -> Void
    let onForget: (WorkspaceInfo) -> Void
    let onForgetDelete: (WorkspaceInfo) -> Void
    let onCreateWorkspace: () -> Void

    @Environment(RepositoryStore.self) private var repositoryStore
    @Environment(RepoWindowManager.self) private var windowManager
    @State private var anchor = PickerAnchor()
    @State private var panel = PickerPanel()
    @State private var rootRepoName: String?
    @State private var isResolvingRootName = false

    private var standardizedRepoPath: String {
        URL(fileURLWithPath: repoPath).standardizedFileURL.path
    }

    private var currentWorkspaceName: String? {
        workspaces.count > 1 ? workspaces.first(where: \.isCurrent)?.name : nil
    }

    var body: some View {
        Button(action: togglePanel) {
            HStack(spacing: 4) {
                Text(rootRepoName ?? URL(fileURLWithPath: repoPath).repositoryDisplayName)
                    .fontWeight(.semibold)
                if let currentWorkspaceName {
                    Text("/")
                        .foregroundStyle(.tertiary)
                    Text(currentWorkspaceName)
                        .fontWeight(.semibold)
                }
                Image(systemName: "chevron.down")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 8)
            .frame(minHeight: 30)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .fixedSize()
        .background(PickerAnchorView(anchor: anchor))
        .help("Switch repository or workspace")
        .accessibilityLabel("Switch Repository or Workspace")
        .onAppear(perform: refresh)
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in
            refresh()
        }
    }

    private func refresh() {
        repositoryStore.reload()
        windowManager.refreshOpenRepoPaths()
        resolveRootRepoName()
    }

    /// A window on a secondary workspace is named after its checkout directory; the button's repo half must show the primary repo instead. The lookup hits the filesystem, so it runs off the main actor; `repoPath` never changes for a window, so a single guarded task cannot be superseded.
    private func resolveRootRepoName() {
        guard rootRepoName == nil, !isResolvingRootName else { return }
        isResolvingRootName = true
        let repoPath = repoPath
        Task { @MainActor in
            let root = await Task.detached { workspacePrimaryRoot(path: repoPath) }.value
            isResolvingRootName = false
            guard let root else { return }
            rootRepoName = URL(fileURLWithPath: root).repositoryDisplayName
        }
    }

    private func togglePanel() {
        guard !panel.isVisible, !panel.wasJustDismissed else {
            panel.dismiss()
            return
        }
        guard let anchorView = anchor.view else { return }
        let sections = [workspaceSection, repositorySection, globalSection].compactMap(\.self)
        let root = PickerPanelRoot(
            placeholder: "Filter",
            actionLabel: "New Workspace",
            onAction: { deferred { onCreateWorkspace() } },
            sections: sections,
            onDismiss: { [weak panel] in panel?.dismiss() }
        )
        panel.show(under: anchorView, size: PickerPanelRoot.idealSize(sections: sections), content: root)
    }

    /// Default-mode work cannot run inside AppKit's event-tracking loop, so window changes wait until the panel is gone.
    private func deferred(_ action: @escaping @MainActor @Sendable () -> Void) {
        RunLoop.main.perform(inModes: [.default]) {
            MainActor.assumeIsolated { action() }
        }
    }

    private var sortedWorkspaces: [WorkspaceInfo] {
        workspaces.sorted {
            if $0.isCurrent != $1.isCurrent {
                return $0.isCurrent
            }
            return $0.name.localizedStandardCompare($1.name) == .orderedAscending
        }
    }

    private var workspaceSection: PickerSection? {
        guard workspaces.count > 1 else { return nil }
        let rows = sortedWorkspaces.map { workspace in
            PickerRow(
                id: "ws-\(workspace.name)",
                searchText: "\(workspace.name) \(workspace.description) \(workspace.isPathResolved ? "" : "path unavailable")",
                height: workspace.description.count > 52 ? 60 : 46,
                action: workspace.isCurrent || !workspace.isPathResolved ? nil : {
                    deferred { onOpenWorkspace(workspace) }
                }
            ) { _ in
                WorkspaceRowView(workspace: workspace)
            }
            .withContextMenu { workspaceContextMenu(workspace) }
        }
        return PickerSection(id: "workspaces", title: "Workspaces", rows: rows)
    }

    @ViewBuilder
    private func workspaceContextMenu(_ workspace: WorkspaceInfo) -> some View {
        if !workspace.isCurrent, workspace.isPathResolved {
            Button("Open in New Window") {
                panel.dismiss()
                deferred { onOpenWorkspace(workspace) }
            }
        }
        if workspace.isPathResolved {
            Button("Copy Path") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(workspace.path, forType: .string)
            }
        }
        if !workspace.isCurrent, workspace.name != "default" {
            Divider()
            Button("Forget") {
                panel.dismiss()
                onForget(workspace)
            }
            if workspace.isPathResolved {
                Button("Forget & Delete from Disk", role: .destructive) {
                    panel.dismiss()
                    onForgetDelete(workspace)
                }
            }
        }
    }

    private var repositorySection: PickerSection? {
        let open = windowManager.openRepoPaths
        let pinned = repositoryStore.paths.filter { !Set(open).contains($0) }
        guard !open.isEmpty || !pinned.isEmpty else { return nil }
        let openRows = open.map { path in
            let isCurrent = path == standardizedRepoPath
            return repoRow(
                path: path,
                icon: isCurrent ? "checkmark" : "macwindow",
                iconTint: isCurrent ? Color.accentColor : .secondary,
                action: isCurrent ? nil : { deferred { windowManager.activateRepo(path) } }
            )
        }
        let pinnedRows = pinned.map { path in
            repoRow(path: path, icon: "pin.fill", iconTint: .secondary) {
                deferred { windowManager.openRepo(path) }
            }
        }
        return PickerSection(id: "repositories", title: "Repositories", rows: openRows + pinnedRows)
    }

    private func repoRow(path: String, icon: String, iconTint: Color, action: (() -> Void)? = nil) -> PickerRow {
        let name = URL(fileURLWithPath: path).repositoryDisplayName
        return PickerRow(
            id: "repo-\(path)",
            searchText: "\(name) \(path)",
            action: action
        ) { _ in
            HStack(spacing: 5) {
                Image(systemName: icon)
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundStyle(iconTint)
                    .frame(width: 14)
                Text(name)
                    .font(.system(size: 13))
                    .lineLimit(1)
                Spacer(minLength: 8)
                Text(path)
                    .font(.system(size: 10))
                    .foregroundStyle(.tertiary)
                    .lineLimit(1)
                    .truncationMode(.head)
                    .frame(maxWidth: 130, alignment: .trailing)
            }
            .padding(.horizontal, 14)
        }
    }

    private var globalSection: PickerSection? {
        PickerSection(id: "global", title: nil, rows: [
            actionRow(id: "repo-list", title: "Repository List…", icon: "list.bullet") {
                deferred { windowManager.showRepoList() }
            },
            actionRow(id: "open-repo", title: "Open Repository…", icon: "folder") {
                deferred { windowManager.openRepositoryPicker() }
            }
        ])
    }

    private func actionRow(id: String, title: String, icon: String, action: @escaping () -> Void) -> PickerRow {
        PickerRow(id: id, searchText: title, height: 28, action: action) { _ in
            HStack(spacing: 5) {
                Image(systemName: icon)
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundStyle(.secondary)
                    .frame(width: 14)
                Text(title)
                    .font(.system(size: 13))
            }
            .padding(.horizontal, 14)
        }
    }
}
