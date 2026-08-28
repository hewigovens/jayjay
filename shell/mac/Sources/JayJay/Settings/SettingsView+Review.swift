import JayJayCore
import SwiftUI

extension SettingsView {
    var reviewSection: some View {
        Section("Review") {
            HStack {
                settingsLabel("Review marks and notes", icon: "checkmark.circle")
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
        }
    }

    var reviewSummaryText: String {
        let marks = reviewSummary.marks == 1 ? "mark" : "marks"
        let notes = reviewSummary.notes == 1 ? "note" : "notes"
        return "\(reviewSummary.marks.formatted()) \(marks), \(reviewSummary.notes.formatted()) \(notes)"
    }

    func clearReviewData() {
        ReviewStore().clearAll()
        reviewSummary = ReviewStoreSummary(marks: 0, notes: 0)
        windowManager.reloadReviewState()
    }
}
