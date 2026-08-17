import Foundation
import JayJayCore

struct RepoGroup: Identifiable {
    let path: String
    let workspaces: [String]
    var id: String {
        path
    }
}

/// A list entry's filesystem identity: where its path really points and, for secondary jj workspaces, the primary repo root, both canonical so entries match across symlinks.
struct RepoPathResolution: Equatable {
    let canonicalPath: String
    let primaryRoot: String?
}

/// Nests recent entries that are secondary jj workspaces under their primary repo's row. Pinned entries always stay top-level, so pinning a workspace promotes it out of its group. Grouping itself is pure; the filesystem lookups it depends on arrive as `resolutions`, so building the list never blocks on a slow volume.
enum RepoListGrouping {
    static func groups(
        pinned: [String],
        recents: [String],
        resolutions: [String: RepoPathResolution]
    ) -> (pinned: [RepoGroup], recent: [RepoGroup]) {
        /// Paths without a resolution fall back to themselves, so entries render flat until their lookups complete.
        func canonical(_ path: String) -> String {
            resolutions[path]?.canonicalPath ?? path
        }

        var ownerByCanonical: [String: String] = [:]
        for path in recents + pinned {
            ownerByCanonical[canonical(path)] = path
        }

        var workspacesByOwner: [String: [String]] = [:]
        var nested: Set<String> = []
        for path in recents {
            guard let root = resolutions[path]?.primaryRoot,
                  root != canonical(path),
                  let owner = ownerByCanonical[root],
                  owner != path
            else { continue }
            workspacesByOwner[owner, default: []].append(path)
            nested.insert(path)
        }

        func group(_ path: String) -> RepoGroup {
            let workspaces = (workspacesByOwner[path] ?? []).sorted {
                displayName($0).localizedStandardCompare(displayName($1)) == .orderedAscending
            }
            return RepoGroup(path: path, workspaces: workspaces)
        }

        return (
            pinned.map(group),
            recents.filter { !nested.contains($0) }.map(group)
        )
    }

    /// Primary-root and symlink lookups hit the filesystem and can each block for a volume timeout on an unreachable path, so they run off the main actor and callers publish the results back into view state.
    static func resolve(paths: [String]) async -> [String: RepoPathResolution] {
        await Task.detached {
            var resolutions: [String: RepoPathResolution] = [:]
            for path in paths {
                resolutions[path] = RepoPathResolution(
                    canonicalPath: canonical(path),
                    primaryRoot: workspacePrimaryRoot(path: path).map(canonical)
                )
            }
            return resolutions
        }.value
    }

    private static func canonical(_ path: String) -> String {
        URL(fileURLWithPath: path).resolvingSymlinksInPath().standardizedFileURL.path
    }

    private static func displayName(_ path: String) -> String {
        URL(fileURLWithPath: path).lastPathComponent
    }
}
