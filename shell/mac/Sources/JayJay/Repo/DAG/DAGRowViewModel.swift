import JayJayCore
import SwiftUI

enum DAGRowSelectionAccent: Equatable {
    case selected
    case compareSource
    case contextTarget

    var color: Color {
        switch self {
            case .selected: .accentColor
            case .compareSource: .orange
            case .contextTarget: .secondary
        }
    }
}

enum DAGRowOutlineState: Equatable {
    case armed
    case hoverTarget
}

enum DAGRowRebaseState: Equatable {
    case none
    case sourceArmed(armedAt: Date?)
    case sourceDragging
    case candidate
    case hoverTarget(previewText: String?)
}

/// Drop-target state for a bookmark/@ chip drag. Unlike a rebase, only the
/// hovered row matters — the dragged chip's own row is untouched.
enum DAGRowBookmarkDropState: Equatable {
    case none
    /// `previewText` is nil until the hover-preview delay elapses, then "Move … here?".
    case hoverTarget(previewText: String?)
}

struct DAGRowViewModel {
    let entry: GraphEntry
    let layout: DAGLayout
    let index: Int
    let colorScheme: ColorScheme
    let selectionAccent: DAGRowSelectionAccent?
    let rebaseState: DAGRowRebaseState
    let bookmarkDropState: DAGRowBookmarkDropState

    init(
        entry: GraphEntry,
        layout: DAGLayout,
        index: Int,
        selectedId: String?,
        compareFromId: String?,
        contextTargetId: String?,
        rebaseDrag: DAGRebaseDragState?,
        rebasePreviewText: String?,
        bookmarkDrag: BookmarkDragState?,
        bookmarkPreviewText: String?,
        colorScheme: ColorScheme
    ) {
        self.entry = entry
        self.layout = layout
        self.index = index
        self.colorScheme = colorScheme

        let rowId = entry.change.selectionRevision
        if selectedId == rowId {
            selectionAccent = .selected
        } else if compareFromId == rowId {
            selectionAccent = .compareSource
        } else if contextTargetId == rowId {
            selectionAccent = .contextTarget
        } else {
            selectionAccent = nil
        }

        rebaseState = Self.rebaseState(
            commitId: entry.change.commitId.id,
            rebaseDrag: rebaseDrag,
            rebasePreviewText: rebasePreviewText
        )
        bookmarkDropState = Self.bookmarkDropState(
            commitId: entry.change.commitId.id,
            bookmarkDrag: bookmarkDrag,
            bookmarkPreviewText: bookmarkPreviewText
        )
    }

    private static func rebaseState(
        commitId: String,
        rebaseDrag: DAGRebaseDragState?,
        rebasePreviewText: String?
    ) -> DAGRowRebaseState {
        if rebaseDrag?.sourceCommitId == commitId {
            switch rebaseDrag?.phase {
                case .armed?: return .sourceArmed(armedAt: rebaseDrag?.armedAt)
                case .dragging?: return .sourceDragging
                default: return .none
            }
        }
        guard rebaseDrag != nil else { return .none }
        if rebaseDrag?.hoveredCommitId == commitId {
            return .hoverTarget(previewText: rebasePreviewText)
        }
        return .candidate
    }

    private static func bookmarkDropState(
        commitId: String,
        bookmarkDrag: BookmarkDragState?,
        bookmarkPreviewText: String?
    ) -> DAGRowBookmarkDropState {
        guard let bookmarkDrag, bookmarkDrag.hoveredCommitId == commitId else { return .none }
        return .hoverTarget(previewText: bookmarkPreviewText)
    }

    var change: ChangeInfo {
        entry.change
    }

    var graphWidth: CGFloat {
        layout.graphWidth
    }

    var descriptionLine: String? {
        let line = change.description.components(separatedBy: "\n").first ?? ""
        let trimmed = line.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    var rowBackground: AnyShapeStyle {
        if isHoverDropTarget {
            return AnyShapeStyle(Color.accentColor.opacity(colorScheme == .dark ? 0.18 : 0.10))
        }
        if isRebaseArmed {
            return AnyShapeStyle(Color.accentColor.opacity(colorScheme == .dark ? 0.14 : 0.08))
        }
        switch selectionAccent {
            case .selected?:
                return AnyShapeStyle(Color.accentColor.opacity(colorScheme == .dark ? 0.18 : 0.10))
            case .compareSource?:
                return AnyShapeStyle(Color.orange.opacity(colorScheme == .dark ? 0.15 : 0.08))
            case .contextTarget?:
                return AnyShapeStyle(Color.secondary.opacity(colorScheme == .dark ? 0.12 : 0.06))
            case nil:
                return AnyShapeStyle(.clear)
        }
    }

    var leadingAccentColor: Color? {
        if isHoverDropTarget {
            return .accentColor
        }
        return selectionAccent?.color
    }

    var outlineState: DAGRowOutlineState? {
        if isHoverDropTarget {
            return .hoverTarget
        }
        if isRebaseArmed {
            return .armed
        }
        return nil
    }

    var isRebaseSource: Bool {
        switch rebaseState {
            case .sourceArmed, .sourceDragging:
                true
            default:
                false
        }
    }

    var isRebaseArmed: Bool {
        if case .sourceArmed = rebaseState {
            return true
        }
        return false
    }

    var isRebaseDragging: Bool {
        if case .sourceDragging = rebaseState {
            return true
        }
        return false
    }

    var isRebaseCandidate: Bool {
        switch rebaseState {
            case .candidate, .hoverTarget:
                true
            default:
                false
        }
    }

    var isRebaseHoverTarget: Bool {
        if case .hoverTarget = rebaseState {
            return true
        }
        return false
    }

    /// Highlighted drop target for either a rebase drag or a bookmark/@ move drag.
    var isHoverDropTarget: Bool {
        if case .hoverTarget = rebaseState { return true }
        if case .hoverTarget = bookmarkDropState { return true }
        return false
    }

    var showsReturnHint: Bool {
        if case let .hoverTarget(previewText) = rebaseState {
            return previewText != nil
        }
        if case let .hoverTarget(previewText) = bookmarkDropState {
            return previewText != nil
        }
        return false
    }

    var dragTargetText: String? {
        switch rebaseState {
            case let .hoverTarget(previewText):
                return previewText ?? "Release to rebase here"
            case .sourceArmed:
                return "Drag to choose a new parent"
            default:
                break
        }
        if case let .hoverTarget(previewText) = bookmarkDropState {
            return previewText ?? "Release to move here"
        }
        return nil
    }

    var scale: CGFloat {
        isRebaseArmed ? 1.01 : 1
    }

    var opacity: Double {
        isRebaseDragging ? 0.56 : 1
    }

    func wiggleAngle(at date: Date) -> Double {
        guard case let .sourceArmed(armedAt) = rebaseState,
              let armedAt,
              date.timeIntervalSince(armedAt) >= 0.12
        else { return 0 }
        return sin(date.timeIntervalSinceReferenceDate * 18) * 1.1
    }
}
