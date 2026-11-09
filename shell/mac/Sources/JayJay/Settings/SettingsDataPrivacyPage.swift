import JayJayCore
import SwiftUI

struct SettingsDataPrivacyPage: View {
    @Environment(AppSettings.self) private var settings
    let windowManager: RepoWindowManager
    @State private var reviewSummary = ReviewStoreSummary(marks: 0, notes: 0)
    @State private var confirmClearReviewData = false
    @State private var pruneOutcome: PruneOutcome?
    @State private var isCheckingPrune = false
    @AppStorage(DebugMode.defaultsKey) private var debugMode = isPrereleaseBuild()

    var body: some View {
        Form {
            Section {
                Toggle(isOn: Binding(
                    get: { settings.sendsAnonymousStats },
                    set: {
                        settings.sendsAnonymousStats = $0
                        AppTelemetry.maybePing(enabled: $0)
                    }
                )) {
                    SettingsLabel(Localized.Telemetry.optIn, icon: "chart.bar")
                }
            } header: {
                Text(Localized.Settings.privacy)
            } footer: {
                Text(Localized.Telemetry.optInDetail)
            }

            Section {
                Toggle(isOn: $debugMode) {
                    SettingsLabel(Localized.Settings.debugMode, icon: "ladybug")
                }
                .onChange(of: debugMode) { setDebugLogging(enabled: debugMode) }
            } header: {
                Text(Localized.Settings.diagnostics)
                    .accessibilityIdentifier(AID.Settings.diagnostics)
            } footer: {
                Text(Localized.Settings.debugModeDetail)
            }

            reviewSection
        }
        .formStyle(.grouped)
    }

    private var reviewSection: some View {
        Section {
            HStack {
                SettingsLabel(Localized.Settings.reviewMarksAndNotes, icon: "checkmark.circle")
                Spacer()
                Text(reviewSummary.text)
                    .foregroundStyle(.secondary)
                Button(Localized.Settings.pruneReview) { checkPrune() }
                    .disabled(reviewSummary.isEmpty || isCheckingPrune)
                    .accessibilityIdentifier(AID.Settings.pruneReviewData)
                Button(Localized.Common.clearEllipsis, role: .destructive) {
                    confirmClearReviewData = true
                }
                .disabled(reviewSummary.isEmpty)
                .accessibilityIdentifier(AID.Settings.clearReviewData)
            }
            .task { reviewSummary = ReviewStore().summary() }
            .alert(Localized.Settings.clearReviewTitle, isPresented: $confirmClearReviewData) {
                Button(Localized.Common.clear, role: .destructive) { clearReviewData() }
                Button(Localized.Common.cancel, role: .cancel) {}
            } message: {
                Text(Localized.Settings.clearReviewMessage(summary: reviewSummary.text))
            }
            .alert(pruneOutcome?.title ?? "", isPresented: isShowingPruneOutcome, presenting: pruneOutcome) { outcome in
                if case .confirm = outcome {
                    Button(Localized.Settings.pruneReviewConfirm, role: .destructive) { pruneReviewData() }
                    Button(Localized.Common.cancel, role: .cancel) {}
                }
            } message: { outcome in
                Text(outcome.message)
            }
        } header: {
            Text(Localized.Settings.reviewData)
        } footer: {
            Text(Localized.Settings.reviewDataDetail)
        }
    }

    private var isShowingPruneOutcome: Binding<Bool> {
        Binding(get: { pruneOutcome != nil }, set: {
            if !$0 {
                pruneOutcome = nil
            }
        })
    }

    /// Open windows can belong to repositories that dropped off the recent list.
    private var pruneRepoPaths: [String] {
        Array(Set(settings.recentRepos + windowManager.openRepoPaths))
    }

    private func checkPrune() {
        isCheckingPrune = true
        let paths = pruneRepoPaths
        Task {
            let result = await Task.detached(priority: .userInitiated) {
                Result { try SharedReviewStore(storePath: nil).prunePreview(repoPaths: paths) }
            }.value
            isCheckingPrune = false
            switch result {
                case let .success(removable):
                    pruneOutcome = removable.isEmpty ? .nothing : .confirm(removable)
                case let .failure(error):
                    pruneOutcome = .failed(error.friendlyDescription)
            }
        }
    }

    private func pruneReviewData() {
        let paths = pruneRepoPaths
        Task {
            let result = await Task.detached(priority: .userInitiated) {
                Result {
                    let store = SharedReviewStore(storePath: nil)
                    _ = try store.prune(repoPaths: paths)
                    return store.summary()
                }
            }.value
            switch result {
                case let .success(remaining):
                    reviewSummary = remaining
                    windowManager.reloadReviewState()
                case let .failure(error):
                    pruneOutcome = .failed(error.friendlyDescription)
            }
        }
    }

    private func clearReviewData() {
        ReviewStore().clearAll()
        reviewSummary = ReviewStoreSummary(marks: 0, notes: 0)
        windowManager.reloadReviewState()
    }
}

private enum PruneOutcome {
    case confirm(ReviewStoreSummary)
    case nothing
    case failed(String)

    var title: String {
        switch self {
            case .confirm: Localized.Settings.pruneReviewTitle
            case .nothing: Localized.Settings.pruneReviewNothingTitle
            case .failed: Localized.Settings.pruneReviewFailedTitle
        }
    }

    var message: String {
        switch self {
            case let .confirm(removable): Localized.Settings.pruneReviewMessage(summary: removable.text)
            case .nothing: Localized.Settings.pruneReviewNothing
            case let .failed(message): message
        }
    }
}

private extension ReviewStoreSummary {
    var isEmpty: Bool {
        marks == 0 && notes == 0
    }

    var text: String {
        Localized.Settings.reviewSummary(marks: Int(marks), notes: Int(notes))
    }
}
