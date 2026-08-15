import Foundation
import JayJayCore

enum WorkspaceSidebarPolicy {
    static func canForget(_ workspace: WorkspaceInfo) -> Bool {
        !workspace.isCurrent
    }

    /// First line of the workspace `@` commit — what this checkout is about.
    static func workingCopySummary(_ workspace: WorkspaceInfo) -> String {
        let description = workspace.description.trimmingCharacters(in: .whitespacesAndNewlines)
        return description.isEmpty ? "—" : description
    }

    static func fileCountVersusParent(_ workspace: WorkspaceInfo) -> String {
        guard let count = workspace.changedFileCount else { return "—" }
        let noun = count == 1 ? "file" : "files"
        return "\(count) \(noun) vs parent"
    }

    static func baselineWorkspace(in workspaces: [WorkspaceInfo]) -> WorkspaceInfo? {
        workspaces.first(where: { $0.name == "default" })
            ?? workspaces.first(where: \.isCurrent)
    }

    static func canCompare(_ workspace: WorkspaceInfo, against baseline: WorkspaceInfo?) -> Bool {
        guard let baseline else { return false }
        return workspace.name != baseline.name
            && !workspace.wcCommitId.isEmpty
            && !baseline.wcCommitId.isEmpty
    }

    static func markingCurrent(_ workspaces: [WorkspaceInfo], name: String) -> [WorkspaceInfo] {
        workspaces.map { workspace in
            WorkspaceInfo(
                name: workspace.name,
                path: workspace.path,
                isCurrent: workspace.name == name,
                wcCommitId: workspace.wcCommitId,
                parentCommitId: workspace.parentCommitId,
                timestampMillis: workspace.timestampMillis,
                changedFileCount: workspace.changedFileCount,
                description: workspace.description,
                pathExists: workspace.pathExists
            )
        }
    }

    static func isSamePath(_ lhs: String, _ rhs: String) -> Bool {
        URL(fileURLWithPath: lhs).standardizedFileURL.path
            == URL(fileURLWithPath: rhs).standardizedFileURL.path
    }

    /// Workspace this window's repo is actually opened on — not the last clicked row.
    static func boundWorkspace(in workspaces: [WorkspaceInfo], repoPath: String) -> WorkspaceInfo? {
        workspaces.first { isSamePath($0.path, repoPath) }
    }

    static func identitySubtitle(_ workspace: WorkspaceInfo) -> String {
        let summary = workingCopySummary(workspace)
        let files = fileCountVersusParent(workspace)
        return summary == "—" ? files : "\(summary) · \(files)"
    }

    /// Click may change the adopted row; `JayJayRepo.open` must wait out pull/push.
    static func shouldDeferRebind(pulling: Bool, pushing: Bool) -> Bool {
        pulling || pushing
    }

    static func identityStatus(pulling: Bool, pushing: Bool, switchPending: Bool) -> String? {
        guard switchPending else { return nil }
        if pulling { return "Waiting for pull…" }
        if pushing { return "Waiting for push…" }
        return nil
    }

    /// A refresh from the still-open path must not rewind the last clicked row.
    static func mergingAdopted(_ incoming: [WorkspaceInfo], current: [WorkspaceInfo]) -> [WorkspaceInfo] {
        guard let adoptedName = current.first(where: \.isCurrent)?.name,
              incoming.contains(where: { $0.name == adoptedName })
        else { return incoming }
        return markingCurrent(incoming, name: adoptedName)
    }
}
