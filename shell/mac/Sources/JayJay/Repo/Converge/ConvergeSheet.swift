import JayJayCore
import SwiftUI

struct ConvergeSheet: View {
    let viewModel: RepoViewModel
    let revision: String
    let onDismiss: () -> Void
    @Environment(\.jayjayTypography) private var typography
    @State private var review = ConvergeReview()

    var body: some View {
        SheetContainer(
            title: Localized.Converge.title,
            cancelLabel: Localized.Common.cancel,
            cancelDisabled: review.isApplying,
            confirmLabel: review.isApplying ? Localized.Converge.applying : Localized.Converge.apply,
            confirmDisabled: !review.canApply,
            confirmAccessibilityIdentifier: AID.Converge.apply,
            footerMessage: review.hint,
            trailingActions: true,
            onCancel: onDismiss,
            onConfirm: { viewModel.applyConvergence(review: review, onSuccess: onDismiss) },
            content: {
                header
                Divider()
                ScrollViewReader { proxy in
                    ScrollView {
                        VStack(alignment: .leading, spacing: 16) {
                            notices
                            if let plan = review.plan {
                                ConvergeVersionsView(review: review, loadedCommitIds: Set(viewModel.graphEntries.map(\.change.commitId.id)))
                                choices
                                    .disabled(review.isLoading || review.isApplying || !review.isCurrent)
                                Divider()
                                ConvergeEffectsView(effects: plan.effects)
                            }
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.trailing, 4)
                    }
                    .onChange(of: review.focusedItem) { _, item in
                        withAnimation { proxy.scrollTo(item) }
                    }
                }
                Divider()
            }
        )
        .background(
            KeyDownMonitor(
                isActive: { review.isCurrent && !review.isApplying },
                yieldsToText: \.isEditable,
                onKeyDown: handleKey
            )
            .frame(width: 0, height: 0)
            .allowsHitTesting(false)
        )
        .frame(width: 780, height: min(760, (NSScreen.main?.visibleFrame.height ?? 900) - 100))
        .interactiveDismissDisabled(review.isApplying)
        .task { viewModel.previewConvergence(revision: revision, review: review) }
        .onDisappear { review.requestId = UUID() }
    }

    @ViewBuilder
    private var header: some View {
        if let plan = review.plan {
            HStack(spacing: 12) {
                let changeId = viewModel.graphEntries.first { $0.change.changeId.id == plan.changeId }?.change.changeId
                    ?? ShortId(id: plan.changeId, shortLen: 8)
                IdentifierText(value: changeId.compact, prefixLength: Int(changeId.shortLen), prefixStyle: .changeId)
                Text(Localized.Converge.versionSummary(count: plan.versions.count))
                    .jayjayFont(11).foregroundStyle(.secondary)
                Spacer()
                if review.unresolvedItems.isEmpty {
                    Label(Localized.Converge.ready, systemImage: "checkmark.circle.fill").foregroundStyle(.green)
                        .jayjayFont(11, weight: .semibold)
                } else {
                    Label(Localized.Converge.unresolvedCount(count: review.unresolvedItems.count), systemImage: "exclamationmark.triangle.fill")
                        .foregroundStyle(.orange)
                        .jayjayFont(11, weight: .semibold)
                }
            }
        }
    }

    @ViewBuilder
    private var notices: some View {
        if review.wasStale {
            Label(Localized.Converge.stale, systemImage: "arrow.clockwise")
                .jayjayFont(12)
                .accessibilityIdentifier(AID.Converge.stale)
        }
        if review.isLoading {
            Text(Localized.Converge.loading).jayjayFont(12).foregroundStyle(.secondary)
        }
        if let error = review.error {
            Text(error).jayjayFont(12).foregroundStyle(.red).textSelection(.enabled)
            Button(Localized.Converge.refresh) {
                viewModel.previewConvergence(revision: revision, review: review)
            }
        }
    }

    private var choices: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text(Localized.Converge.choices).jayjayFont(13, weight: .semibold)
                Spacer()
                Text(Localized.Converge.decidedAutomatically(decided: "\(review.automaticCount)", total: "\(ConvergeItem.allCases.count)"))
                    .jayjayFont(11).foregroundStyle(.secondary)
            }
            ForEach(review.cardItems, id: \.self) { item in
                card(item).id(item)
            }
            sameItems
        }
    }

    @ViewBuilder
    private func card(_ item: ConvergeItem) -> some View {
        switch item {
            case .contents:
                ConvergeContentsCard(review: review, repo: viewModel.repo, diffStore: viewModel.diffStore)
            case .description:
                ConvergeChoiceCard(item: item, review: review) {
                    Button(Localized.Converge.edit) { review.editDescription() }
                        .buttonStyle(.plain)
                        .fontWeight(.medium)
                        .foregroundStyle(Color.accentColor)
                        .disabled(review.customDescription != nil)
                        .accessibilityIdentifier(AID.Converge.editDescription)
                } content: {
                    if review.customDescription != nil {
                        TextEditor(text: Binding(get: { review.customDescription ?? "" }, set: { review.customDescription = $0 }))
                            .jayjayFont(12)
                            .frame(height: 80)
                            .padding(6)
                            .accessibilityLabel(Localized.Converge.description)
                            .accessibilityIdentifier(AID.Converge.customDescription)
                    } else {
                        ConvergeVersionValues(item: item, review: review)
                    }
                }
            case .author, .parents:
                ConvergeChoiceCard(item: item, review: review) {
                    EmptyView()
                } content: {
                    ConvergeVersionValues(item: item, review: review)
                }
        }
    }

    @ViewBuilder
    private var sameItems: some View {
        let items = review.sameItems
        if !items.isEmpty {
            VStack(spacing: 0) {
                ForEach(Array(items.enumerated()), id: \.element) { index, item in
                    if index > 0 {
                        Divider()
                    }
                    HStack(alignment: .firstTextBaseline, spacing: 10) {
                        Image(systemName: "checkmark").jayjayFont(11, weight: .bold).foregroundStyle(.green)
                        Text(item.title).jayjayFont(12, weight: .semibold)
                            .frame(width: 80 * typography.interfaceScale, alignment: .leading)
                        Text(sameValue(item))
                            .jayjayFont(12, design: item == .parents ? .monospaced : .default)
                            .lineLimit(1)
                            .textSelection(.enabled)
                        Spacer()
                        Text(Localized.Converge.same(count: review.versions.count)).jayjayFont(11).foregroundStyle(.secondary)
                    }
                    .padding(.horizontal, 10)
                    .padding(.vertical, 7)
                }
            }
            .background(Color(nsColor: .textBackgroundColor), in: RoundedRectangle(cornerRadius: 7))
            .overlay(RoundedRectangle(cornerRadius: 7).stroke(Color.primary.opacity(0.12)))
        }
    }

    private func sameValue(_ item: ConvergeItem) -> String {
        guard let version = review.versions.first else { return "" }
        if item == .description {
            return version.subject.isEmpty ? Localized.Converge.noDescription : version.subject
        }
        return version.value(of: item) ?? ""
    }

    private func handleKey(_ event: NSEvent) -> Bool {
        switch [AppShortcut.acceptLeftHunk, .acceptRightHunk, .previousHunk, .nextHunk].first(where: { $0.matches(event) }) {
            case .acceptLeftHunk: review.useVersion(at: 0)
            case .acceptRightHunk: review.useVersion(at: 1)
            case .previousHunk: review.moveFocus(by: -1)
            case .nextHunk: review.moveFocus(by: 1)
            default: return false
        }
        return true
    }
}
