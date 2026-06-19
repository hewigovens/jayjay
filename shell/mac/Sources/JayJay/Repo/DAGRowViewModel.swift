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

struct DAGRowViewModel {
    let entry: GraphEntry
    let layout: DAGLayout
    let index: Int
    let colorScheme: ColorScheme
    let selectionAccent: DAGRowSelectionAccent?
    let rebaseState: DAGRowRebaseState

    init(
        entry: GraphEntry,
        layout: DAGLayout,
        index: Int,
        selectedId: String?,
        compareFromId: String?,
        contextTargetId: String?,
        rebaseDrag: DAGRebaseDragState?,
        rebasePreviewText: String?,
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

        if rebaseDrag?.sourceCommitId == entry.change.commitId.id {
            switch rebaseDrag?.phase {
                case .pressing?:
                    rebaseState = .none
                case .armed?:
                    rebaseState = .sourceArmed(armedAt: rebaseDrag?.armedAt)
                case .dragging?:
                    rebaseState = .sourceDragging
                case nil:
                    rebaseState = .none
            }
        } else if rebaseDrag != nil {
            if rebaseDrag?.hoveredCommitId == entry.change.commitId.id {
                rebaseState = .hoverTarget(previewText: rebasePreviewText)
            } else {
                rebaseState = .candidate
            }
        } else {
            rebaseState = .none
        }
    }

    var change: ChangeInfo {
        entry.change
    }

    var graphWidth: CGFloat {
        min(160, CGFloat(max(layout.maxLanes(), 1)) * laneWidth + 8)
    }

    var descriptionLine: String? {
        let line = change.description.components(separatedBy: "\n").first ?? ""
        let trimmed = line.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? nil : trimmed
    }

    var rowBackground: AnyShapeStyle {
        if isRebaseHoverTarget {
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
        if isRebaseHoverTarget {
            return .accentColor
        }
        return selectionAccent?.color
    }

    var outlineState: DAGRowOutlineState? {
        if isRebaseHoverTarget {
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

    var showsReturnHint: Bool {
        if case let .hoverTarget(previewText) = rebaseState {
            return previewText != nil
        }
        return false
    }

    var dragTargetText: String? {
        switch rebaseState {
            case let .hoverTarget(previewText):
                previewText ?? "Release to rebase here"
            case .sourceArmed:
                "Drag to choose a new parent"
            default:
                nil
        }
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
