import JayJayCore
import SwiftUI

extension RepoContentView {
    func selectWorkspace(_ workspace: WorkspaceInfo) {
        if !workspace.pathExists {
            viewModel.info = "Workspace path is gone: \(workspace.path)"
            return
        }
        if workspace.isCurrent { return }
        viewModel.adoptWorkspaceAppearance(workspace)
        if WorkspaceSidebarPolicy.shouldDeferRebind(
            pulling: viewModel.isPullingInFlight,
            pushing: viewModel.isPushingInFlight
        ) {
            viewModel.pendingWorkspacePath = workspace.path
            commitWorkspaceGraph(workspace)
            return
        }
        viewModel.pendingWorkspacePath = nil
        windowManager.selectWorkspace(workspace.path, rebind: onRebindWorkspace)
        commitWorkspaceGraph(workspace)
    }

    func flushDeferredWorkspaceSwitch() {
        guard !viewModel.hasSyncInFlight,
              let path = viewModel.pendingWorkspacePath
        else { return }
        viewModel.pendingWorkspacePath = nil
        windowManager.selectWorkspace(path, rebind: onRebindWorkspace)
    }

    /// Sidebar and identity already follow the click. Graph, files, and VoiceOver wait so rapid clicks do not bounce.
    func commitWorkspaceGraph(_ workspace: WorkspaceInfo) {
        workspaceCommitTask?.cancel()
        let generation = viewModel.workspaceSwitchGeneration
        workspaceCommitTask = Task { @MainActor in
            do {
                try await Task.sleep(for: .milliseconds(160))
            } catch {
                return
            }
            guard viewModel.workspaceSwitchGeneration == generation else { return }
            if !workspace.wcCommitId.isEmpty {
                viewModel.select(changeId: workspace.wcCommitId)
                dagRevealRequest = DAGRevealRequest(changeId: workspace.wcCommitId)
            }
            AccessibilityNotification.Announcement("Workspace \(workspace.name)").post()
        }
    }
}
