import JayJayCore
import SwiftUI

struct PullRequestImportSheet: View {
    static var title: String {
        Localized.Repo.newWorkspaceFromPullRequest
    }

    let viewModel: RepoViewModel
    let onDismiss: () -> Void
    let onOpen: (String) -> Void

    @State private var url = ""
    @State private var preview: PullRequestImportPreview?
    @State private var workspaceName = ""
    @State private var workspaceDest = ""
    @State private var errorMessage: String?
    @State private var importsAgain = false

    var body: some View {
        SheetContainer(
            title: Self.title,
            subtitle: viewModel.prHostName.map { Localized.Repo.createWorkspaceFromPullRequestOn(host: $0) }
                ?? Localized.Repo.createAWorkspaceFromAPull,
            cancelLabel: Localized.Common.cancel,
            confirmLabel: confirmLabel,
            confirmDisabled: confirmDisabled,
            confirmAccessibilityIdentifier: preview == nil
                ? AID.PullRequestImport.resolveButton : AID.PullRequestImport.createButton,
            onCancel: cancel,
            onConfirm: confirm,
            content: {
                if let preview {
                    previewContent(preview)
                } else {
                    urlContent
                }
            }
        )
        .frame(width: 440)
    }

    private var existingWorkspace: PullRequestImportWorkspace? {
        importsAgain ? nil : preview?.existingWorkspace
    }

    private var confirmLabel: String {
        if preview == nil {
            return viewModel.isResolvingPullRequest ? Localized.Repo.resolvingEllipsis : Localized.Repo.resolve
        }
        if existingWorkspace != nil {
            return Localized.Repo.openWorkspace
        }
        return viewModel.isImportingPullRequest ? Localized.Repo.creatingEllipsis : Localized.Repo.fetchCreateWorkspace
    }

    private var confirmDisabled: Bool {
        if preview == nil {
            return viewModel.isResolvingPullRequest || url.isEmpty
        }
        if existingWorkspace != nil {
            return false
        }
        return viewModel.isImportingPullRequest || viewModel.isResolvingPullRequest || workspaceDest.isEmpty
            || !isValidWorkspaceName(name: workspaceName)
    }

    private var urlContent: some View {
        VStack(alignment: .leading, spacing: 10) {
            TextField(Localized.Repo.pullRequestUrl, text: $url)
                .textFieldStyle(.roundedBorder)
                .jayjayFont(13, design: .monospaced)
                .accessibilityIdentifier(AID.PullRequestImport.urlField)
                .onSubmit(confirm)
            errorText
        }
    }

    private func previewContent(_ preview: PullRequestImportPreview) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 6) {
                stateBadge(preview.pullRequest.state)
                Text("#\(preview.pullRequest.number)")
                    .jayjayFont(12, weight: .semibold)
                Text(preview.pullRequest.title)
                    .jayjayFont(12)
                    .lineLimit(1)
                    .truncationMode(.tail)
            }
            VStack(alignment: .leading, spacing: 6) {
                summaryRow(label: Localized.Menu.repository, value: "\(preview.pullRequest.host) · \(preview.pullRequest.baseRepo)")
                summaryRow(
                    label: Localized.Repo.remote,
                    value: preview.remote.name,
                    note: preview.remote.exists ? Localized.Repo.existingRemote : Localized.Repo.willBeAdded
                )
                summaryRow(label: Localized.Repo.bookmark, value: preview.remote.bookmark)
                summaryRow(label: Localized.Repo.head, value: String(preview.headCommitId.prefix(8)))
            }
            if let existingWorkspace {
                HStack(spacing: 8) {
                    summaryRow(label: Localized.Repo.workspace, value: existingWorkspace.name, note: Localized.Repo.alreadyImported)
                    Button(Localized.Repo.importAgain) { importsAgain = true }
                        .buttonStyle(.link)
                        .jayjayFont(11)
                }
            } else {
                TextField(Localized.Repo.workspaceName, text: $workspaceName)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(13, design: .monospaced)
                TextField(Localized.Repo.location, text: $workspaceDest)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(13, design: .monospaced)
            }
            errorText
        }
    }

    private func summaryRow(label: String, value: String, note: String? = nil) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Text(label)
                .jayjayFont(11, weight: .semibold)
                .foregroundStyle(.secondary)
                .frame(width: 70, alignment: .leading)
            Text(value)
                .jayjayFont(11, design: .monospaced)
                .lineLimit(1)
                .truncationMode(.middle)
            if let note {
                Text(note)
                    .jayjayFont(10)
                    .foregroundStyle(.tertiary)
            }
            Spacer(minLength: 0)
        }
    }

    private func stateBadge(_ state: PrState) -> some View {
        let label: String
        let color: Color
        switch state {
            case .open:
                label = Localized.Common.open
                color = .green
            case .closed:
                label = Localized.Repo.closed
                color = .red
            case .merged:
                label = Localized.Repo.merged
                color = .purple
        }
        return Text(label)
            .jayjayFont(9, weight: .semibold)
            .padding(.horizontal, 5)
            .padding(.vertical, 1)
            .background(color.opacity(0.18), in: Capsule())
            .foregroundStyle(color)
    }

    @ViewBuilder
    private var errorText: some View {
        if let errorMessage {
            Text(errorMessage)
                .jayjayFont(11)
                .foregroundStyle(.red)
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityIdentifier(AID.PullRequestImport.error)
        }
    }

    private func cancel() {
        viewModel.cancelPullRequestImport()
        if !viewModel.isImportingPullRequest {
            onDismiss()
        }
    }

    private func confirm() {
        if preview == nil {
            resolve()
        } else if let existingWorkspace {
            onDismiss()
            onOpen(existingWorkspace.dest)
        } else {
            importWorkspace()
        }
    }

    private func resolve() {
        errorMessage = nil
        viewModel.resolvePullRequestImport(url: url, onSuccess: { preview in
            workspaceName = preview.workspace.name
            workspaceDest = preview.workspace.dest
            self.preview = preview
        }, onFailure: { error in
            errorMessage = error.friendlyDescription
        })
    }

    /// A failed import means the PR or repository changed underneath the preview, so show the current state with the error.
    private func refreshPreview() {
        viewModel.resolvePullRequestImport(url: url, onSuccess: { preview = $0 }, onFailure: { _ in })
    }

    private func importWorkspace() {
        guard let preview else { return }
        errorMessage = nil
        viewModel.importPullRequest(
            PullRequestImportRequest(
                url: url, previewedHeadCommitId: preview.headCommitId,
                workspaceName: workspaceName, workspaceDest: workspaceDest
            ),
            onSuccess: { createdPath in
                onDismiss()
                onOpen(createdPath)
            },
            onFailure: { error in
                errorMessage = error.friendlyDescription
                refreshPreview()
            }
        )
    }
}
