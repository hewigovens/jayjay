import AppKit
import SwiftUI

/// Anchored to the line's marker so the popover scrolls with the content instead of floating over it.
@MainActor
func presentReviewNotePopover(
    from gutterTextView: DiffGutterTextView,
    at rect: NSRect,
    notes: [DiffReviewNoteSummary],
    actions: any DiffGutterNoteActions
) {
    let popover = NSPopover()
    popover.behavior = .transient
    let content = ReviewNotePopover(
        notes: notes,
        editNote: { [weak popover] id in
            popover?.performClose(nil)
            actions.editNote(id: id)
        },
        resolveNote: { [weak popover] id in
            popover?.performClose(nil)
            actions.resolveNote(id: id)
        },
        deleteNote: { [weak popover] id in
            popover?.performClose(nil)
            actions.deleteNote(id: id)
        }
    )
    let hosting = NSHostingController(rootView: content)
    // Without this the hosting view stretches to the popover's default size and shows a large empty pane under the content.
    hosting.sizingOptions = .preferredContentSize
    popover.contentViewController = hosting
    // The gutter retains the popover (transient popovers are otherwise released) and closes any previous one so markers never stack popovers.
    gutterTextView.activeNotePopover?.performClose(nil)
    gutterTextView.activeNotePopover = popover
    // Anchor below the marked line (the gutter view is flipped, so .maxY is the visual bottom) — a side edge centers the popover vertically on the line and covers the code the note is about.
    popover.show(relativeTo: rect, of: gutterTextView, preferredEdge: .maxY)
}

struct ReviewNotePopover: View {
    let notes: [DiffReviewNoteSummary]
    let editNote: (String) -> Void
    let resolveNote: (String) -> Void
    let deleteNote: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 6) {
                Image(systemName: "text.bubble.fill")
                    .font(.system(size: 11))
                    .foregroundStyle(.orange)
                Text(notes.count == 1 ? "Review Note" : "\(notes.count) Review Notes")
                    .font(.system(size: 12, weight: .semibold))
                Spacer(minLength: 0)
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)

            Divider()

            rows
        }
        .frame(width: 320)
        .focusEffectDisabled()
    }

    @ViewBuilder
    private var rows: some View {
        let list = VStack(alignment: .leading, spacing: 0) {
            ForEach(Array(notes.enumerated()), id: \.element.id) { index, note in
                if index > 0 {
                    Divider().padding(.leading, 14)
                }
                ReviewNotePopoverRow(
                    note: note,
                    editNote: editNote,
                    resolveNote: resolveNote,
                    deleteNote: deleteNote
                )
            }
        }
        if notes.count > 3 {
            ScrollView {
                list
            }
            .frame(height: 320)
        } else {
            list
        }
    }
}

private struct ReviewNotePopoverRow: View {
    let note: DiffReviewNoteSummary
    let editNote: (String) -> Void
    let resolveNote: (String) -> Void
    let deleteNote: (String) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            if note.isResolved {
                Label("Resolved", systemImage: "checkmark.circle.fill")
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundStyle(.secondary)
            }
            Text(note.body)
                .font(.system(size: 12))
                .foregroundStyle(note.isResolved ? AnyShapeStyle(.secondary) : AnyShapeStyle(.primary))
                .lineLimit(6)
                .fixedSize(horizontal: false, vertical: true)
                .textSelection(.enabled)

            HStack(spacing: 4) {
                if !note.isResolved {
                    actionButton("Edit", systemImage: "pencil") { editNote(note.id) }
                    actionButton("Resolve", systemImage: "checkmark.circle") { resolveNote(note.id) }
                }
                Spacer(minLength: 0)
                actionButton("Delete", systemImage: "trash", role: .destructive) { deleteNote(note.id) }
            }
            .padding(.top, 2)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
    }

    private func actionButton(
        _ title: String,
        systemImage: String,
        role: ButtonRole? = nil,
        action: @escaping () -> Void
    ) -> some View {
        Button(role: role, action: action) {
            Label(title, systemImage: systemImage)
                .font(.system(size: 11, weight: .medium))
                .padding(.horizontal, 7)
                .padding(.vertical, 3.5)
                .contentShape(RoundedRectangle(cornerRadius: 5))
        }
        .buttonStyle(.plain)
        .foregroundStyle(role == .destructive ? AnyShapeStyle(.red.opacity(0.85)) : AnyShapeStyle(.secondary))
        .background(.quaternary.opacity(0.5), in: RoundedRectangle(cornerRadius: 5))
    }
}
