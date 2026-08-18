import AppKit
import JayJayCore
import SwiftUI

struct DAGRow: View {
    @Environment(\.colorScheme) var colorScheme
    let viewModel: DAGRowViewModel
    var prHostName: String?
    var onMoveBookmarkToRev: ((String, String) -> Void)?
    var onPushBookmark: ((String) -> Void)?
    var onOpenPRForBookmark: ((String) -> Void)?
    var onDeleteBookmark: ((String) -> Void)?
    var conflictedBookmarkNames: Set<String> = []
    var onBookmarkDragChanged: ((String, String, DragGesture.Value) -> Void)?
    var onBookmarkDragEnded: ((String, DragGesture.Value) -> Void)?

    /// Non-private: read by the DAGRow+GraphColumn / +Refs extensions.
    var change: ChangeInfo {
        viewModel.change
    }

    var body: some View {
        if viewModel.isRebaseArmed {
            TimelineView(.animation) { timeline in
                rowBody(wiggleAngle: viewModel.wiggleAngle(at: timeline.date))
            }
        } else {
            rowBody(wiggleAngle: 0)
        }
    }

    private func rowBody(wiggleAngle: Double) -> some View {
        HStack(alignment: .top, spacing: 0) {
            graphColumn
                .frame(width: viewModel.graphWidth)

            VStack(alignment: .leading, spacing: 5) {
                refsRow
                    .lineLimit(1)

                if let descriptionLine = viewModel.descriptionLine {
                    Text(descriptionLine)
                        .jayjayFont(13, weight: .medium).lineLimit(1)
                        .help(change.description)
                } else {
                    Text("(no description)").jayjayFont(13).foregroundStyle(.tertiary)
                }

                HStack(spacing: 6) {
                    CommitAvatar(email: change.author.email, size: 14)
                    Text(change.author.name)
                    Text(relativeDate(change.author.timestampMillis)).foregroundStyle(.secondary)
                }
                .jayjayFont(10).lineLimit(1).truncationMode(.tail).foregroundStyle(.secondary)
            }
            .padding(.vertical, dagRowVerticalPadding)
            .padding(.trailing, 10)
            Spacer(minLength: 0)
        }
        .padding(.leading, dagRowLeadingPadding)
        .background(viewModel.rowBackground)
        .rotationEffect(.degrees(wiggleAngle))
        .scaleEffect(viewModel.scale)
        .opacity(viewModel.opacity)
        .overlay(alignment: .leading) {
            if let accent = viewModel.leadingAccentColor {
                RoundedRectangle(cornerRadius: 2, style: .continuous)
                    .fill(accent)
                    .frame(width: 3)
            }
        }
        .overlay {
            switch viewModel.outlineState {
                case .hoverTarget?:
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .stroke(Color.accentColor, lineWidth: 2)
                        .padding(.vertical, 2)
                case .armed?:
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .stroke(
                            Color.accentColor.opacity(0.7),
                            style: StrokeStyle(lineWidth: 1.5, dash: [5, 4])
                        )
                        .padding(.vertical, 2)
                case nil:
                    EmptyView()
            }
        }
        .overlay(alignment: .trailing) {
            if let dragTargetText = viewModel.dragTargetText {
                dragTargetBubble(dragTargetText)
                    .padding(.trailing, 10)
            }
        }
    }

    private static let relativeFormatter = RelativeDateTimeFormatter()

    /// Relative time the change last moved; the change id shown above is the stable identifier.
    private func relativeDate(_ millis: Int64) -> String {
        let date = Date(timeIntervalSince1970: Double(millis) / 1000)
        let now = Date()
        // Floor to whole minutes: a per-second count ("19s, 20s, …") on fresh changes is
        // distracting, and a clock-skewed future timestamp then reads as "1 minute ago".
        let reference = min(date, now.addingTimeInterval(-60))
        return Self.relativeFormatter.localizedString(for: reference, relativeTo: now)
    }

    private func dragTargetBubble(_ text: String) -> some View {
        HStack(spacing: 6) {
            Text(text)
                .jayjayFont(10, weight: .medium)
                .lineLimit(1)
            if viewModel.showsReturnHint {
                hintChip("return")
            }
            hintChip("esc")
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(
            viewModel.isRebaseHoverTarget ? Color.accentColor.opacity(0.14) : Color.clear,
            in: Capsule()
        )
        .glassEffect(in: Capsule())
        .overlay(
            Capsule()
                .stroke(Color.accentColor.opacity(viewModel.isRebaseHoverTarget ? 0.5 : 0.2), lineWidth: 1)
        )
    }

    private func hintChip(_ text: String) -> some View {
        Text(text.uppercased())
            .jayjayFont(8, weight: .semibold, design: .monospaced)
            .foregroundStyle(.secondary)
            .padding(.horizontal, 4)
            .padding(.vertical, 2)
            .background(Color.primary.opacity(0.06), in: Capsule())
    }
}
