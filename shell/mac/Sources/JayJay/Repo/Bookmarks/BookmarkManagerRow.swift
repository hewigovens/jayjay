import JayJayCore
import SwiftUI

protocol BookmarkManagerRowActions {
    func filterBookmark(_ bookmark: BookmarkInfo)
    func diffBookmark(_ bookmark: BookmarkInfo)
    func deleteBookmark(_ bookmark: BookmarkInfo)
    func forgetBookmark(_ bookmark: BookmarkInfo)
    func pushBookmark(_ bookmark: BookmarkInfo)
    func resolveBookmarkConflict(_ bookmark: BookmarkInfo)
    func openPullRequest(for bookmark: BookmarkInfo)
    func trackBookmark(_ bookmark: BookmarkInfo, remote: String)
}

struct BookmarkManagerRow<Actions: BookmarkManagerRowActions>: View {
    let bookmark: BookmarkInfo
    let prHostName: String?
    let actions: Actions

    @Environment(\.colorScheme) private var colorScheme

    private var canOpenPR: Bool {
        bookmark.isTrackingRemote && !bookmark.isDeleted && !isTrunkBookmark(bookmark.name)
    }

    private var canDiffBookmark: Bool {
        !bookmark.isDeleted && !bookmark.isConflicted && !bookmark.changeId.id.isEmpty
            && !isTrunkBookmark(bookmark.name)
    }

    private var remoteSuffix: String {
        guard !bookmark.hasLocalTarget, let first = bookmark.availableRemotes.first else {
            return ""
        }
        return "@\(first)"
    }

    private var pullRequestLabel: String {
        if let prHostName {
            return "Pull Request on \(prHostName)"
        }
        return "Pull Request"
    }

    var body: some View {
        HStack(spacing: 10) {
            statusIcon
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(bookmark.name + remoteSuffix)
                        .jayjayFont(13, weight: .medium, design: .monospaced)
                        .lineLimit(1)
                    if bookmark.isDeleted {
                        badge("deleted", color: .red)
                    }
                    if bookmark.isConflicted {
                        badge("conflicted", color: .orange)
                    }
                    if !bookmark.hasLocalTarget, !bookmark.isDeleted {
                        badge("remote-only", color: .blue)
                    } else if bookmark.hasLocalTarget, !bookmark.isTrackingRemote, !bookmark.isDeleted {
                        badge("local", color: .secondary)
                    }
                }
                HStack(spacing: 6) {
                    if !bookmark.description.isEmpty {
                        Text(bookmark.description)
                            .jayjayFont(11)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                    ForEach(bookmark.remoteTargets, id: \.remote) { target in
                        remoteBadge(target)
                    }
                }
            }
            Spacer()
            if !bookmark.changeId.id.isEmpty {
                Text(bookmark.changeId.highlighted(scheme: colorScheme))
                    .jayjayFont(11, design: .monospaced)
            }
        }
        .padding(.vertical, 4)
        .contextMenu {
            if !bookmark.isDeleted, !bookmark.changeId.id.isEmpty {
                Button { actions.filterBookmark(bookmark) } label: {
                    Label("Filter in DAG", systemImage: "line.3.horizontal.decrease.circle")
                }
                if canDiffBookmark {
                    Button { actions.diffBookmark(bookmark) } label: {
                        Label("Diff Bookmark", systemImage: "arrow.left.arrow.right")
                    }
                }
            }
            if bookmark.isConflicted {
                Button { actions.resolveBookmarkConflict(bookmark) } label: {
                    Label("Resolve conflict (set to @)", systemImage: "arrow.triangle.merge")
                }
            }
            if !bookmark.hasLocalTarget, !bookmark.isDeleted {
                ForEach(bookmark.availableRemotes, id: \.self) { remote in
                    Button { actions.trackBookmark(bookmark, remote: remote) } label: {
                        Label("Track \(bookmark.name)@\(remote)", systemImage: "arrow.down.circle")
                    }
                }
            } else if bookmark.isTrackingRemote, !bookmark.isDeleted {
                Button { actions.pushBookmark(bookmark) } label: {
                    Label("Push", systemImage: "arrow.up.circle")
                }
            }
            if canOpenPR {
                Button { actions.openPullRequest(for: bookmark) } label: {
                    Label(pullRequestLabel, systemImage: "arrow.up.right.square")
                }
            }
            if bookmark.isDeleted {
                Divider()
                Button { actions.forgetBookmark(bookmark) } label: {
                    Label("Forget (clean up)", systemImage: "bookmark.slash")
                }
            } else if bookmark.hasLocalTarget {
                Divider()
                Button(role: .destructive) { actions.deleteBookmark(bookmark) } label: {
                    Label("Delete", systemImage: "trash")
                }
            }
        }
    }

    private var statusIcon: some View {
        Group {
            if bookmark.isDeleted {
                Image(systemName: "bookmark.slash.fill")
                    .foregroundStyle(.red)
            } else if bookmark.isConflicted {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.orange)
            } else if !bookmark.hasLocalTarget {
                Image(systemName: "cloud")
                    .foregroundStyle(.blue)
            } else if bookmark.isTrackingRemote {
                Image(systemName: "cloud.fill")
                    .foregroundStyle(.blue)
            } else {
                Image(systemName: "bookmark.fill")
                    .foregroundStyle(.secondary)
            }
        }
        .jayjayFont(12)
        .frame(width: 16)
    }

    private func badge(_ text: String, color: Color) -> some View {
        Text(text)
            .jayjayFont(9, weight: .semibold)
            .foregroundStyle(color)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(color.opacity(0.12), in: Capsule())
    }

    /// Sync state of one tracked remote ref, relative to the local bookmark: green when in sync, amber when one side is ahead, red when diverged.
    @ViewBuilder
    private func remoteBadge(_ target: RemoteBookmarkTarget) -> some View {
        switch target.status {
            case .synced:
                badge("\(target.remote) ✓", color: .green)
                    .help("In sync with \(target.remote).")
            case .ahead:
                badge("ahead of \(target.remote)", color: .orange)
                    .help(remoteHelp(target, "Local is ahead of \(target.remote) — push to update it."))
            case .behind:
                badge("behind \(target.remote)", color: .orange)
                    .help(remoteHelp(target, "\(target.remote) has newer commits — fetch to catch up."))
            case .diverged:
                badge("diverged from \(target.remote)", color: .red)
                    .help(remoteHelp(target, "\(bookmark.name) and \(target.remote) have each moved on."))
            @unknown default:
                EmptyView()
        }
    }

    /// Tooltip: the action sentence plus where the remote ref currently sits.
    private func remoteHelp(_ target: RemoteBookmarkTarget, _ lead: String) -> String {
        guard !target.changeId.isEmpty else { return lead }
        let summary = target.description.isEmpty ? "" : " — \(target.description)"
        return "\(lead) \(target.remote) is at \(String(target.changeId.prefix(8)))\(summary)."
    }
}
