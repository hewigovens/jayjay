import JayJayCore
import SwiftUI

extension ChangeCommand {
    var title: String {
        changeCommandTitle(command: self)
    }

    var accelerator: KeyEquivalent? {
        changeCommandAccelerator(command: self)?.first.map { KeyEquivalent($0) }
    }

    var shortcut: AppShortcut? {
        changeCommandShortcutId(command: self).flatMap(AppShortcut.init(rawValue:))
    }

    var systemImage: String {
        switch self {
            case .newChange: "plus.circle"
            case .newChangeBefore: "arrow.down.circle"
            case .newChangeAfter: "arrow.up.circle"
            case .edit: "pencil.circle"
            case .describe: "square.and.pencil"
            case .squashIntoParent: "arrow.down.left.circle"
            case .moveChangesToWorkingCopy: "tray.and.arrow.down"
            case .rebaseOntoTrunk: "arrow.uturn.up"
            case .fix: "wand.and.stars"
            case .createBookmark: "bookmark"
            case .createTag: "tag"
            case .showEvolution: "clock.arrow.circlepath"
            case .duplicate: "doc.on.doc"
            case .absorb: "arrow.down.to.line"
            case .revert: "arrow.uturn.backward"
            case .abandon: "trash"
        }
    }

    func perform(on change: ChangeInfo, actions: (any DAGActions)?, onRequest: ((DAGRequest) -> Void)?) {
        if let request = request(for: change) {
            onRequest?(request)
        } else if let actions {
            perform(rev: change.selectionRevision, on: actions)
        }
    }

    private func request(for change: ChangeInfo) -> DAGRequest? {
        let rev = change.selectionRevision
        switch self {
            case .describe: return .editDescription(rev: rev, description: change.description)
            case .rebaseOntoTrunk: return .rebaseOntoTrunk(change)
            case .createBookmark: return .createBookmark(rev: rev)
            case .createTag: return .createTag(rev: change.commitId.id)
            case .abandon: return .abandon(rev: rev)
            default: return nil
        }
    }

    private func perform(rev: String, on actions: any DAGActions) {
        switch self {
            case .newChange: actions.newChange(parent: rev, message: "")
            case .newChangeBefore: actions.insertChange(rev: rev, position: .before)
            case .newChangeAfter: actions.insertChange(rev: rev, position: .after)
            case .duplicate: actions.duplicate(rev: rev)
            case .revert: actions.revertChange(rev: rev)
            case .showEvolution: actions.showEvolog(rev: rev)
            default: rewrite(rev: rev, on: actions)
        }
    }

    private func rewrite(rev: String, on actions: any DAGActions) {
        switch self {
            case .edit: actions.edit(rev: rev)
            case .squashIntoParent: actions.squash(rev: rev)
            case .moveChangesToWorkingCopy: actions.squash(rev: rev, into: "@")
            case .fix: actions.fix(revs: [rev])
            case .absorb: actions.absorb(rev: rev)
            default: break
        }
    }
}

extension SelectionCommand {
    func title(count: Int) -> String {
        selectionCommandTitle(command: self, count: UInt32(count))
    }

    var accelerator: KeyEquivalent? {
        selectionCommandAccelerator(command: self)?.first.map { KeyEquivalent($0) }
    }

    var systemImage: String {
        switch self {
            case .merge: "arrow.triangle.merge"
            case .squash: "arrow.down.left.circle"
            case .parallelize: "arrow.triangle.branch"
            case .fix: "wand.and.stars"
            case .rebaseOntoTrunk: "arrow.uturn.up"
            case .abandon: "trash"
        }
    }

    /// Merge takes `revisions` as the parents in order, so each first-parent choice is its own call.
    func perform(on revisions: [String], actions: (any DAGActions)?, onRequest: ((DAGRequest) -> Void)?) {
        switch self {
            case .merge: actions?.merge(parents: revisions)
            case .squash: onRequest?(.squashSelection(revisions: revisions))
            case .parallelize: actions?.parallelize(revs: revisions)
            case .fix: actions?.fix(revs: revisions)
            case .rebaseOntoTrunk: onRequest?(.rebaseSelectionOntoTrunk(revisions: revisions))
            case .abandon: onRequest?(.abandonSelection(revisions: revisions))
        }
    }
}
