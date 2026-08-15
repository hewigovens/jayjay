import JayJayCore
import SwiftUI

extension RepoContentView {
    var workspaceSidebar: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Workspaces")
                    .jayjayFont(11, weight: .semibold)
                    .foregroundStyle(.secondary)
                Spacer()
                Button {
                    settings.workspaceSidebarVisible = false
                } label: {
                    Image(systemName: "sidebar.leading")
                }
                .buttonStyle(.plain)
                .help("Hide Workspaces")
                .accessibilityLabel("Hide Workspaces")
                .accessibilityIdentifier(AID.Workspace.toggle)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)

            if viewModel.workspaces.count > 12 {
                TextField("Search workspaces", text: $workspaceSearch)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(11)
                    .padding(.horizontal, 10)
                    .padding(.bottom, 6)
                    .accessibilityIdentifier(AID.Workspace.search)
            }

            if viewModel.workspaces.isEmpty, viewModel.isLoading {
                Text("Loading…")
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(10)
            } else if let error = viewModel.error, viewModel.workspaces.isEmpty {
                VStack(alignment: .leading, spacing: 6) {
                    Text(error)
                        .jayjayFont(11)
                        .foregroundStyle(.red)
                        .accessibilityIdentifier(AID.Workspace.error)
                    Button("Retry") { viewModel.refresh() }
                        .accessibilityIdentifier(AID.Workspace.retry)
                }
                .padding(10)
            } else {
                workspaceList
            }

            Button {
                modal = .workspaceCreate
            } label: {
                Label("New Workspace", systemImage: "plus")
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .buttonStyle(.plain)
            .padding(10)
            .accessibilityIdentifier(AID.Workspace.newWorkspace)
        }
        .accessibilityIdentifier(AID.Workspace.sidebar)
    }

    var workspaceSidebarRail: some View {
        VStack(spacing: 8) {
            Button {
                settings.workspaceSidebarVisible = true
            } label: {
                Image(systemName: "sidebar.leading")
            }
            .buttonStyle(.plain)
            .help("Show Workspaces")
            .accessibilityLabel("Show Workspaces")
            .accessibilityIdentifier(AID.Workspace.toggle)

            Text(currentWorkspaceInitial)
                .jayjayFont(13, weight: .semibold)
                .help(viewModel.workspaces.first(where: \.isCurrent)?.name ?? "Workspace")
        }
        .frame(width: 28)
        .padding(.vertical, 8)
        .accessibilityIdentifier(AID.Workspace.rail)
    }

    private var workspaceList: some View {
        VStack(spacing: 0) {
            List(selection: workspaceSelection) {
                Section("Default") {
                    ForEach(pinnedWorkspaces, id: \.name) { workspace in
                        workspaceRow(workspace)
                    }
                }
                if !filteredNamedWorkspaces.isEmpty {
                    Section("Workspaces") {
                        ForEach(filteredNamedWorkspaces, id: \.name) { workspace in
                            workspaceRow(workspace)
                        }
                    }
                }
            }
            .listStyle(.sidebar)
            if namedWorkspaces.isEmpty {
                Text("Create a workspace to work in parallel.")
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 10)
                    .padding(.bottom, 6)
                    .accessibilityIdentifier(AID.Workspace.emptyHint)
            }
        }
    }

    private var pinnedWorkspaces: [WorkspaceInfo] {
        let pin = viewModel.workspaces.contains(where: { $0.name == "default" })
            ? "default"
            : viewModel.workspaces.first(where: \.isCurrent)?.name
        return viewModel.workspaces.filter { $0.name == pin }
    }

    private var namedWorkspaces: [WorkspaceInfo] {
        let pinned = Set(pinnedWorkspaces.map(\.name))
        return viewModel.workspaces.filter { !pinned.contains($0.name) }
    }

    private var filteredNamedWorkspaces: [WorkspaceInfo] {
        let query = workspaceSearch.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !query.isEmpty else { return namedWorkspaces }
        return namedWorkspaces.filter { $0.name.localizedCaseInsensitiveContains(query) }
    }

    private var currentWorkspaceInitial: String {
        let name = viewModel.workspaces.first(where: \.isCurrent)?.name ?? "?"
        return String(name.prefix(1)).uppercased()
    }

    private var workspaceSelection: Binding<String?> {
        Binding(
            get: { viewModel.workspaces.first(where: \.isCurrent)?.name },
            set: { name in
                guard let name,
                      let workspace = viewModel.workspaces.first(where: { $0.name == name }),
                      !workspace.isCurrent
                else { return }
                selectWorkspace(workspace)
            }
        )
    }

    @ViewBuilder
    private func workspaceRow(_ workspace: WorkspaceInfo) -> some View {
        WorkspaceSidebarRow(
            workspace: workspace,
            trunkName: workspace.name == pinnedWorkspaces.first?.name ? viewModel.trunkBookmarkName : nil
        )
        .tag(workspace.name)
        .listRowInsets(EdgeInsets(top: 4, leading: 8, bottom: 4, trailing: 8))
        .opacity(workspace.pathExists ? 1 : 0.45)
        .accessibilityIdentifier(AID.Workspace.row(workspace.name))
        .help(workspaceHelp(workspace))
        .contextMenu {
            Button("Show Changes") {
                viewModel.showWorkspaceChanges(workspace)
            }
            .accessibilityIdentifier(AID.Workspace.showChanges)
            if WorkspaceSidebarPolicy.canCompare(
                workspace,
                against: WorkspaceSidebarPolicy.baselineWorkspace(in: viewModel.workspaces)
            ), let baseline = WorkspaceSidebarPolicy.baselineWorkspace(in: viewModel.workspaces) {
                Button("Compare with \(baseline.name)") {
                    viewModel.compareWorkspace(workspace, against: baseline)
                }
            }
            Button("Open in New Window") {
                windowManager.openRepo(workspace.path)
            }
            .accessibilityIdentifier(AID.Workspace.openInNewWindow)
            Button("Forget Workspace", role: .destructive) {
                forgetWorkspace = workspace
            }
            .disabled(!WorkspaceSidebarPolicy.canForget(workspace))
            .accessibilityIdentifier(AID.Workspace.forget)
        }
        .onTapGesture { selectWorkspace(workspace) }
        .confirmationDialog(
            "Forget \(forgetWorkspace?.name ?? "workspace")?",
            isPresented: Binding(
                get: { forgetWorkspace?.name == workspace.name },
                set: { if !$0 { forgetWorkspace = nil } }
            ),
            titleVisibility: .visible
        ) {
            Button("Forget", role: .destructive) {
                if let target = forgetWorkspace, WorkspaceSidebarPolicy.canForget(target) {
                    viewModel.workspaceForget(name: target.name)
                    settings.removeRecentRepo(target.path)
                }
                forgetWorkspace = nil
            }
            Button("Cancel", role: .cancel) {
                forgetWorkspace = nil
            }
        } message: {
            Text("Removes the workspace from jj. Files on disk are kept.")
        }
    }

    private func workspaceHelp(_ workspace: WorkspaceInfo) -> String {
        let description = workspace.description.isEmpty ? "—" : workspace.description
        return "\(workspace.name)\n\(workspace.path)\n\(description)"
    }
}

private struct WorkspaceSidebarRow: View {
    let workspace: WorkspaceInfo
    let trunkName: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 6) {
                if workspace.isCurrent {
                    Image(systemName: "circle.fill")
                        .font(.system(size: 6))
                        .accessibilityIdentifier(AID.Workspace.currentIndicator(workspace.name))
                }
                Text(workspace.name)
                    .jayjayFont(12, weight: workspace.isCurrent ? .semibold : .regular)
                    .lineLimit(1)
                if let trunkName, !trunkName.isEmpty {
                    Text(trunkName)
                        .jayjayFont(10, weight: .medium)
                        .padding(.horizontal, 5)
                        .padding(.vertical, 1)
                        .background(.quaternary, in: Capsule())
                }
                Spacer()
                Text(relativeTime)
                    .jayjayFont(10)
                    .foregroundStyle(.secondary)
                    .monospacedDigit()
            }
            Text(WorkspaceSidebarPolicy.workingCopySummary(workspace))
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .lineLimit(2)
            Text(WorkspaceSidebarPolicy.fileCountVersusParent(workspace))
                .jayjayFont(10)
                .foregroundStyle(.tertiary)
        }
        .frame(minHeight: 32, alignment: .leading)
        .contentShape(Rectangle())
    }

    private var relativeTime: String {
        guard let millis = workspace.timestampMillis else { return "—" }
        let date = Date(timeIntervalSince1970: Double(millis) / 1000)
        let now = Date()
        let reference = min(date, now.addingTimeInterval(-60))
        return Self.relativeFormatter.localizedString(for: reference, relativeTo: now)
    }

    private static let relativeFormatter: RelativeDateTimeFormatter = {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter
    }()
}
