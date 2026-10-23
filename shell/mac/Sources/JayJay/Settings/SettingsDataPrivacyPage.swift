import JayJayCore
import SwiftUI

struct SettingsDataPrivacyPage: View {
    @Environment(AppSettings.self) private var settings
    let windowManager: RepoWindowManager
    @State private var reviewSummary = ReviewStoreSummary(marks: 0, notes: 0)
    @State private var confirmClearReviewData = false
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
                Text(reviewSummaryText)
                    .foregroundStyle(.secondary)
                Button(Localized.Common.clearEllipsis, role: .destructive) {
                    confirmClearReviewData = true
                }
                .disabled(reviewSummary.marks == 0 && reviewSummary.notes == 0)
                .accessibilityIdentifier(AID.Settings.clearReviewData)
            }
            .task { reviewSummary = ReviewStore().summary() }
            .alert(Localized.Settings.clearReviewTitle, isPresented: $confirmClearReviewData) {
                Button(Localized.Common.clear, role: .destructive) { clearReviewData() }
                Button(Localized.Common.cancel, role: .cancel) {}
            } message: {
                Text(Localized.Settings.clearReviewMessage(summary: reviewSummaryText))
            }
        } header: {
            Text(Localized.Settings.reviewData)
        } footer: {
            Text(Localized.Settings.reviewDataDetail)
        }
    }

    private var reviewSummaryText: String {
        Localized.Settings.reviewSummary(marks: Int(reviewSummary.marks), notes: Int(reviewSummary.notes))
    }

    private func clearReviewData() {
        ReviewStore().clearAll()
        reviewSummary = ReviewStoreSummary(marks: 0, notes: 0)
        windowManager.reloadReviewState()
    }
}
