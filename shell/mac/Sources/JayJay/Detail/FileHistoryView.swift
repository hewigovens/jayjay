import JayJayCore
import SwiftUI

struct FileHistoryView: View {
    let history: [ChangeInfo]
    let path: String
    let onSelectChange: (String) -> Void
    let onDismiss: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            if history.isEmpty {
                ContentUnavailableView(
                    "No History",
                    systemImage: "clock",
                    description: Text("No revisions modified this file.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(history, id: \.changeId) { change in
                    Button {
                        onSelectChange(change.changeId)
                    } label: {
                        VStack(alignment: .leading, spacing: 4) {
                            HStack(spacing: 6) {
                                Text(String(change.changeId.prefix(8)))
                                    .jayjayFont(11, weight: .semibold, design: .monospaced)
                                    .foregroundStyle(Color.accentColor)
                                Text(change.author)
                                    .jayjayFont(11)
                                    .foregroundStyle(.secondary)
                                Spacer()
                                Text(formatTimestamp(change.timestampMillis))
                                    .jayjayFont(10)
                                    .foregroundStyle(.tertiary)
                            }
                            if !change.description.isEmpty {
                                Text(change.description.components(separatedBy: "\n").first ?? "")
                                    .jayjayFont(12, weight: .medium)
                                    .lineLimit(1)
                            } else {
                                Text("(no description)")
                                    .jayjayFont(12)
                                    .foregroundStyle(.tertiary)
                            }
                        }
                        .padding(.vertical, 2)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                }
                .listStyle(.plain)
            }
        }
    }

    private var header: some View {
        HStack {
            Image(systemName: "clock.arrow.trianglehead.counterclockwise.rotate.90")
                .foregroundStyle(.secondary)
            Text("History: \(path)")
                .jayjayFont(13, weight: .semibold, design: .monospaced)
                .lineLimit(1)
            Spacer()
            Text("\(history.count) revisions")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button {
                onDismiss()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .help("Close history view")
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
    }

    private func formatTimestamp(_ millis: Int64) -> String {
        let date = Date(timeIntervalSince1970: Double(millis) / 1000)
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: Date())
    }
}
