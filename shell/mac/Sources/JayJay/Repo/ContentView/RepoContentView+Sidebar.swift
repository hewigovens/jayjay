import JayJayCore
import SwiftUI

extension RepoContentView {
    var sidebar: some View {
        VStack(spacing: 0) {
            if showRevsetFilter {
                VStack(spacing: 6) {
                    HStack(spacing: 6) {
                        if let previousAncestorFilter {
                            Button {
                                revsetDraft = previousAncestorFilter
                                applyRevset()
                            } label: {
                                Image(systemName: "arrow.left")
                            }
                            .buttonStyle(.plain)
                            .help("Back to previous filter")
                            .accessibilityLabel("Back to previous filter")
                        }
                        TextField("Revset expression", text: $revsetDraft)
                            .textFieldStyle(.roundedBorder).jayjayFont(12, design: .monospaced)
                            .onSubmit { applyRevset() }
                            .keyboardFocusInput(.revsetInput)
                        Button { applyRevset() } label: {
                            Image(systemName: "arrow.right.circle.fill").foregroundStyle(.secondary)
                        }
                        .buttonStyle(.plain).disabled(revsetDraft == viewModel.revset)
                        Button {
                            revsetDraft = ""
                            applyRevset()
                        } label: {
                            Image(systemName: "xmark.circle.fill").foregroundStyle(.tertiary)
                        }
                        .buttonStyle(.plain)
                        .help("Reset to default")
                    }
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 6) {
                            ForEach(RevsetExpressions.filterPresets, id: \.id) { preset in
                                revsetChip(preset.label, revset: preset.revset)
                            }
                        }
                    }
                }
                .padding(.horizontal, 12).padding(.vertical, 8)
                Divider()
            }
            if let name = viewModel.pendingPushBookmark {
                pushFollowUpBanner(name)
                Divider()
            }
            DAGView(
                entries: viewModel.graphEntries,
                layout: viewModel.dagLayout,
                capabilities: viewModel.selectionCapabilities,
                graphGeneration: viewModel.graphGeneration,
                selectedId: viewModel.selectedChangeId,
                selectedIds: viewModel.selectedChangeIds,
                compareFromId: viewModel.compareFromId,
                actions: viewModel,
                onRequest: { handleDAGRequest($0) },
                activePane: Bindable(keyboardFocus).activePane,
                revealRequest: dagRevealRequest,
                prHostName: viewModel.prHostName,
                conflictedBookmarkNames: viewModel.conflictedBookmarkNames,
                workspacesByName: viewModel.workspacesByName
            )
            if shouldShowCommitBox {
                Divider()
                CommitBox(
                    description: viewModel.workingCopyDescription,
                    summary: $viewModel.commitSummaryDraft,
                    details: $viewModel.commitDescriptionDraft,
                    onSaveDescription: { viewModel.describeWorkingCopy(message: $0) },
                    onCommit: {
                        await viewModel.commit(message: $0, manageSubmodules: settings.enableGitSubmoduleSupport)
                    },
                    onGenerateMessage: { await viewModel.generateCommitMessage() },
                    aiProvider: viewModel.aiProvider
                )
            }
        }
    }

    func pushFollowUpBanner(_ name: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: "bookmark.fill").foregroundStyle(.green).jayjayFont(11)
            Text("Moved").jayjayFont(11).foregroundStyle(.secondary)
            Text(name).jayjayFont(11, weight: .medium, design: .monospaced).lineLimit(1)
            Spacer()
            Button("Push") { viewModel.confirmPendingPush() }
                .controlSize(.small)
                .disabled(viewModel.isPushingInFlight)
            Button {
                viewModel.dismissPendingPush()
            } label: {
                Image(systemName: "xmark").jayjayFont(10)
            }
            .buttonStyle(.plain).foregroundStyle(.secondary)
            .help("Dismiss")
        }
        .padding(.horizontal, 12).padding(.vertical, 6)
        .glassEffect(in: RoundedRectangle(cornerRadius: 8))
    }

    func revsetChip(_ label: String, revset: String) -> some View {
        Button {
            revsetDraft = revset
            applyRevset()
        } label: {
            Text(label)
                .jayjayFont(11, weight: .medium)
                .padding(.horizontal, 10)
                .padding(.vertical, 4)
                .background(
                    viewModel.revset == revset
                        ? AnyShapeStyle(Color.accentColor.opacity(0.2))
                        : AnyShapeStyle(Color.primary.opacity(0.06)),
                    in: Capsule()
                )
        }
        .buttonStyle(.plain)
    }

    func applyRevset() {
        previousAncestorFilter = nil
        let t = revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        revsetDraft = t.isEmpty ? RepoViewModel.buildDefaultRevset() : t
        viewModel.applyRevset(revsetDraft)
    }

    private var shouldShowCommitBox: Bool {
        viewModel.selectedChange?.info.isWorkingCopy == true
    }
}
