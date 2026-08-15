import JayJayCore
import SwiftUI

struct WorkspaceIdentityBar: View {
    let workspace: WorkspaceInfo
    var trunkName: String?
    var waitingCaption: String?

    var body: some View {
        HStack(spacing: 8) {
            Capsule()
                .fill(Color.accentColor)
                .frame(width: 3, height: 12)
            Image(systemName: "square.on.square")
                .jayjayFont(11, weight: .semibold)
                .foregroundStyle(.secondary)
            Text(workspace.name)
                .jayjayFont(12, weight: .semibold)
                .lineLimit(1)
            if let trunkName, !trunkName.isEmpty {
                Text(trunkName)
                    .jayjayFont(10, weight: .medium)
                    .padding(.horizontal, 5)
                    .padding(.vertical, 1)
                    .background(.quaternary, in: Capsule())
            }
            Text(waitingCaption ?? WorkspaceSidebarPolicy.identitySubtitle(workspace))
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .lineLimit(1)
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .frame(maxWidth: .infinity, alignment: .leading)
        .fixedSize(horizontal: false, vertical: true)
        .background(.bar)
        .accessibilityElement(children: .combine)
        .accessibilityIdentifier(AID.Workspace.identity)
        .accessibilityLabel("Workspace \(workspace.name)")
        .accessibilityValue(waitingCaption ?? WorkspaceSidebarPolicy.identitySubtitle(workspace))
        .accessibilityAddTraits(.updatesFrequently)
    }
}
