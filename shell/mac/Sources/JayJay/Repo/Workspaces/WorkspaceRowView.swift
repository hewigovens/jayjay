import JayJayCore
import SwiftUI

/// Two-line picker row for a workspace: name, conflict badge, and recency up top; change id, description, and changed-file count below.
struct WorkspaceRowView: View {
    let workspace: WorkspaceInfo

    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(alignment: .firstTextBaseline, spacing: 5) {
                Image(systemName: workspace.isCurrent ? "checkmark" : "folder")
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundStyle(workspace.isCurrent ? Color.accentColor : .secondary)
                    .frame(width: 14)
                Text("\(workspace.name):")
                    .font(.system(size: 13, weight: .semibold))
                    .lineLimit(1)
                Text(workspace.changeId.highlighted(scheme: colorScheme, maxChars: 8))
                    .font(.system(size: 11, design: .monospaced))
                if workspace.hasConflict {
                    Text("conflict")
                        .font(.system(size: 9, weight: .semibold))
                        .padding(.horizontal, 5).padding(.vertical, 1)
                        .background(.red.opacity(0.15), in: Capsule())
                }
                Spacer(minLength: 8)
                Text(Date.relativeLabel(millis: workspace.timestamp))
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
            }
            HStack(alignment: .top, spacing: 6) {
                descriptionText
                    .font(.system(size: 11))
                    .lineLimit(2)
                    .truncationMode(.tail)
                    .multilineTextAlignment(.leading)
                Spacer(minLength: 8)
                if workspace.filesChanged > 0 {
                    Text("\(workspace.filesChanged) file\(workspace.filesChanged == 1 ? "" : "s")")
                        .font(.system(size: 10))
                        .foregroundStyle(.secondary)
                        .layoutPriority(1)
                }
            }
            .padding(.leading, 19)
        }
        .padding(.horizontal, 14)
    }

    @ViewBuilder
    private var descriptionText: some View {
        if !workspace.isPathResolved {
            Text("Path unavailable — Forget to clean up")
                .foregroundStyle(.orange)
        } else if workspace.description.isEmpty {
            Text("(no description)")
                .foregroundStyle(.tertiary)
        } else {
            Text(workspace.description)
                .foregroundStyle(.secondary)
        }
    }
}
