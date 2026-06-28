import JayJayCore
import SwiftUI

extension ChangeDetailView {
    var staleReviewNotesSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            ForEach(staleOrOrphanedReviewNotes, id: \.note.id) { item in
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Image(systemName: item.status == .orphaned ? "link.badge.plus" : "exclamationmark.triangle")
                        .foregroundStyle(.orange)
                    Text(reviewNoteStatusLabel(item.status))
                        .jayjayFont(11, weight: .semibold)
                        .foregroundStyle(.orange)
                    Text("\(item.note.path):\(item.note.line)")
                        .jayjayFont(11, design: .monospaced)
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Text(item.note.body)
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                    Spacer()
                    Button("Resolve") {
                        reviewStore.resolveNote(id: item.note.id)
                        refreshReviewState()
                    }
                    .buttonStyle(.link)
                }
            }
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 8)
        .background(.orange.opacity(0.07))
    }

    private func reviewNoteStatusLabel(_ status: NoteStatus) -> String {
        switch status {
            case .stale: "Stale"
            case .orphaned: "Orphaned"
            case .resolved: "Resolved"
            case .current: "Current"
        }
    }
}
