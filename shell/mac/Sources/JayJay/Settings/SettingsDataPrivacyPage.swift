import JayJayCore
import SwiftUI

struct SettingsDataPrivacyPage: View {
    @Environment(AppSettings.self) private var settings
    let windowManager: RepoWindowManager
    @State private var reviewSummary = ReviewStoreSummary(marks: 0, notes: 0)
    @State private var confirmClearReviewData = false

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
                    SettingsLabel("Share anonymous build and OS stats", icon: "chart.bar")
                }
            } header: {
                Text("Privacy")
            } footer: {
                Text("No repository, file, or command data is sent.")
            }

            reviewSection
        }
        .formStyle(.grouped)
    }

    private var reviewSection: some View {
        Section {
            HStack {
                SettingsLabel("Review marks and notes", icon: "checkmark.circle")
                Spacer()
                Text(reviewSummaryText)
                    .foregroundStyle(.secondary)
                Button("Clear…", role: .destructive) {
                    confirmClearReviewData = true
                }
                .disabled(reviewSummary.marks == 0 && reviewSummary.notes == 0)
                .accessibilityIdentifier(AID.Settings.clearReviewData)
            }
            .task { reviewSummary = ReviewStore().summary() }
            .alert("Clear all review marks and notes?", isPresented: $confirmClearReviewData) {
                Button("Clear", role: .destructive) { clearReviewData() }
                Button("Cancel", role: .cancel) {}
            } message: {
                Text("Removes \(reviewSummaryText) for every repository. This cannot be undone.")
            }
        } header: {
            Text("Review Data")
        } footer: {
            Text("Review marks and notes are stored on this Mac. Clearing them affects every repository and cannot be undone.")
        }
    }

    private var reviewSummaryText: String {
        let marks = reviewSummary.marks == 1 ? "mark" : "marks"
        let notes = reviewSummary.notes == 1 ? "note" : "notes"
        return "\(reviewSummary.marks.formatted()) \(marks), \(reviewSummary.notes.formatted()) \(notes)"
    }

    private func clearReviewData() {
        ReviewStore().clearAll()
        reviewSummary = ReviewStoreSummary(marks: 0, notes: 0)
        windowManager.reloadReviewState()
    }
}
