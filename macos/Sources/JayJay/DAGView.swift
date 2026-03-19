import SwiftUI
import JayJayBindings

struct DAGView: View {
    let entries: [GraphEntry]
    let selectedId: String?
    let onSelect: (String) -> Void
    var onNew: ((String) -> Void)?
    var onSquash: ((String) -> Void)?
    var onAbandon: ((String) -> Void)?

    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        Group {
            if entries.isEmpty {
                ContentUnavailableView(
                    "No Changes Matched",
                    systemImage: "line.3.horizontal.decrease.circle",
                    description: Text("Try a broader revset or refresh the repository.")
                )
            } else {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(entries.enumerated()), id: \.element.change.changeId) { index, entry in
                            DAGRow(
                                entry: entry,
                                isSelected: selectedId == entry.change.changeId,
                                isLast: index == entries.count - 1,
                                colorScheme: colorScheme
                            )
                            .contentShape(Rectangle())
                            .onTapGesture {
                                onSelect(entry.change.changeId)
                            }
                            .contextMenu {
                                Button("New child change") {
                                    onNew?(entry.change.changeId)
                                }
                                Button("Squash into parent") {
                                    onSquash?(entry.change.changeId)
                                }
                                Divider()
                                Button("Abandon", role: .destructive) {
                                    onAbandon?(entry.change.changeId)
                                }
                            }
                        }
                    }
                    .padding(.vertical, 6)
                }
                .background(
                    LinearGradient(
                        colors: [
                            Color.primary.opacity(colorScheme == .dark ? 0.03 : 0.015),
                            .clear
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
            }
        }
    }
}

struct DAGRow: View {
    let entry: GraphEntry
    let isSelected: Bool
    let isLast: Bool
    let colorScheme: ColorScheme

    private var change: ChangeInfo { entry.change }

    var body: some View {
        HStack(alignment: .top, spacing: 0) {
            // Graph column
            graphColumn
                .frame(width: 28)

            // Content
            VStack(alignment: .leading, spacing: 5) {
                HStack(spacing: 6) {
                    Text(shortId(change.changeId))
                        .jayjayFont(11, weight: .semibold, design: .monospaced)
                        .foregroundStyle(change.isWorkingCopy ? Color.accentColor : .secondary)

                    if change.isWorkingCopy {
                        tag("@", tint: .accentColor.opacity(0.18))
                    }

                    if change.hasConflict {
                        tag("conflict", tint: .red.opacity(0.18))
                    }

                    ForEach(change.bookmarks, id: \.self) { bookmark in
                        tag(bookmark, tint: .primary.opacity(0.08))
                    }
                }

                if !change.description.isEmpty {
                    Text(change.description.components(separatedBy: "\n").first ?? "")
                        .jayjayFont(13, weight: .medium)
                        .lineLimit(1)
                } else {
                    Text("(no description)")
                        .jayjayFont(13)
                        .foregroundStyle(.tertiary)
                }

                HStack(spacing: 6) {
                    Text(change.author)
                    Text(shortId(change.commitId))
                        .foregroundStyle(.secondary)
                }
                .jayjayFont(10, design: .monospaced)
                .lineLimit(1)
                .truncationMode(.tail)
                .foregroundStyle(.secondary)
            }
            .padding(.trailing, 10)

            Spacer(minLength: 0)
        }
        .padding(.vertical, 8)
        .padding(.leading, 6)
        .background(selectionBackground)
        .overlay(alignment: .leading) {
            if isSelected {
                RoundedRectangle(cornerRadius: 2, style: .continuous)
                    .fill(Color.accentColor)
                    .frame(width: 3)
            }
        }
    }

    private var graphColumn: some View {
        GeometryReader { geo in
            let midX: CGFloat = 14
            let nodeY: CGFloat = 10
            let height = geo.size.height

            Canvas { ctx, size in
                // Draw edge line below node
                if !isLast {
                    let linePath = Path { p in
                        p.move(to: CGPoint(x: midX, y: nodeY + 5))
                        p.addLine(to: CGPoint(x: midX, y: height))
                    }
                    let hasIndirect = entry.edges.contains { $0.edgeType == .indirect }
                    if hasIndirect {
                        ctx.stroke(linePath, with: .color(.secondary.opacity(0.25)),
                                   style: StrokeStyle(lineWidth: 1, dash: [3, 3]))
                    } else {
                        ctx.stroke(linePath, with: .color(.secondary.opacity(0.25)),
                                   style: StrokeStyle(lineWidth: 1))
                    }
                }

                // Draw node
                let nodeRect = CGRect(x: midX - 4, y: nodeY - 4, width: 8, height: 8)
                let nodePath = Path(ellipseIn: nodeRect)
                if change.isWorkingCopy {
                    ctx.fill(nodePath, with: .color(.accentColor))
                } else if change.isEmpty {
                    ctx.stroke(nodePath, with: .color(.secondary.opacity(0.5)),
                               style: StrokeStyle(lineWidth: 1.5))
                } else if change.hasConflict {
                    ctx.fill(nodePath, with: .color(.red))
                } else {
                    ctx.fill(nodePath, with: .color(.secondary.opacity(0.5)))
                }
            }
        }
    }

    private var selectionBackground: some ShapeStyle {
        if isSelected {
            AnyShapeStyle(Color.accentColor.opacity(colorScheme == .dark ? 0.18 : 0.10))
        } else {
            AnyShapeStyle(.clear)
        }
    }

    @ViewBuilder
    private func tag(_ title: String, tint: Color) -> some View {
        Text(title)
            .jayjayFont(9, weight: .semibold)
            .padding(.horizontal, 5)
            .padding(.vertical, 2)
            .background(tint, in: Capsule())
    }

    private func shortId(_ id: String) -> String {
        String(id.prefix(12))
    }
}
