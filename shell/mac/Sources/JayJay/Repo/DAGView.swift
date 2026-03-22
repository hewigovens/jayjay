import JayJayCore
import SwiftUI

struct DAGView: View {
    let entries: [GraphEntry]
    let selectedId: String?
    let actions: (any DAGActions)?
    var onAbandon: ((String) -> Void)?
    var onCreateBookmark: ((String) -> Void)?

    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        Group {
            if entries.isEmpty {
                ContentUnavailableView(
                    "No Changes Matched",
                    systemImage: "line.3.horizontal.decrease.circle",
                    description: Text("Try a broader revset or refresh.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                let layout = DAGLayout(entries: entries)
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(entries.enumerated()), id: \.element.change.changeId) { index, entry in
                            DAGRow(
                                entry: entry, layout: layout, index: index,
                                isSelected: selectedId == entry.change.changeId,
                                isLast: index == entries.count - 1,
                                colorScheme: colorScheme
                            )
                            .contentShape(Rectangle())
                            .onTapGesture { actions?.select(changeId: entry.change.changeId) }
                            .contextMenu {
                                Button("Edit (switch to)") { actions?.edit(rev: entry.change.changeId) }
                                Button("New child change") { actions?.newChange(
                                    parent: entry.change.changeId,
                                    message: ""
                                ) }
                                Button("Cherry-pick (graft)") { actions?.graft(rev: entry.change.changeId) }
                                Button("Duplicate") { actions?.duplicate(rev: entry.change.changeId) }
                                Button("Squash into parent") { actions?.squash(rev: entry.change.changeId) }
                                if let sel = selectedId, sel != entry.change.changeId {
                                    Button("Squash selected into this") {
                                        actions?.squash(rev: sel, into: entry.change.changeId)
                                    }
                                    Button("Merge with selected") {
                                        actions?.merge(parents: [sel, entry.change.changeId])
                                    }
                                }
                                Button("Create bookmark here...") { onCreateBookmark?(entry.change.changeId) }
                                Divider()
                                Button("Abandon", role: .destructive) { onAbandon?(entry.change.changeId) }
                            }
                        }
                    }
                    .padding(.vertical, 6)
                }
                .background(
                    LinearGradient(
                        colors: [Color.primary.opacity(colorScheme == .dark ? 0.03 : 0.015), .clear],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
            }
        }
    }
}
