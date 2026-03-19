import SwiftUI
import JayJayBindings

struct DAGView: View {
    let changes: [ChangeInfo]
    @Binding var selectedId: String?

    var body: some View {
        List(selection: $selectedId) {
            ForEach(changes, id: \.changeId) { change in
                DAGRow(change: change)
                    .tag(change.changeId)
            }
        }
        .listStyle(.inset(alternatesRowBackgrounds: true))
    }
}

struct DAGRow: View {
    let change: ChangeInfo

    var body: some View {
        HStack(spacing: 8) {
            // Graph node indicator
            Circle()
                .fill(change.isWorkingCopy ? Color.accentColor : Color.secondary)
                .frame(width: 8, height: 8)

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(shortId(change.changeId))
                        .font(.caption.monospaced().bold())
                        .foregroundStyle(change.isWorkingCopy ? Color.accentColor : .primary)

                    if !change.bookmarks.isEmpty {
                        ForEach(change.bookmarks, id: \.self) { bookmark in
                            Text(bookmark)
                                .font(.caption2)
                                .padding(.horizontal, 4)
                                .padding(.vertical, 1)
                                .background(.tint.opacity(0.15), in: .capsule)
                        }
                    }

                    if change.isWorkingCopy {
                        Text("@")
                            .font(.caption.bold())
                            .foregroundStyle(Color.accentColor)
                    }
                }

                if !change.description.isEmpty {
                    Text(change.description.components(separatedBy: "\n").first ?? "")
                        .font(.callout)
                        .lineLimit(1)
                } else {
                    Text("(no description)")
                        .font(.callout)
                        .foregroundStyle(.tertiary)
                }

                Text("\(change.author) \(shortId(change.commitId))")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            Spacer()
        }
        .padding(.vertical, 2)
    }

    private func shortId(_ id: String) -> String {
        String(id.prefix(12))
    }
}
