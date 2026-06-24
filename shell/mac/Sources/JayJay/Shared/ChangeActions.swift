import Foundation
import JayJayCore

let trunkBookmarkNames: Set<String> = ["main", "master", "trunk"]

/// Matches bare "main" as well as remote-qualified forms like "main@origin".
func isTrunkBookmark(_ name: String) -> Bool {
    let bare = name.split(separator: "@").first.map(String.init) ?? name
    return trunkBookmarkNames.contains(bare)
}

protocol ChangeActions: AnyObject {
    func select(changeId: String?)
    func describeChange(rev: String, message: String)
    func restoreFiles(rev: String, paths: [String])
    func deleteFiles(paths: [String])
    func ignoreAndUntrack(paths: [String])
    func split(rev: String, paths: [String], message: String, parallel: Bool)
    func moveToWorkingCopy(rev: String, paths: [String])
    func applyDiffSelection(
        rev: String,
        destination: DiffEditDestination,
        selections: [DiffEditFileSelection],
        message: String,
        ignoreWhitespace: Bool
    )
    func resolveUseOurs(rev: String, path: String)
    func resolveUseTheirs(rev: String, path: String)
    func resolveInEditor(rev: String, path: String, tool: String)
}

protocol DAGActions: AnyObject {
    func select(changeId: String?)
    func edit(rev: String)
    func newChange(parent: String, message: String)
    func graft(rev: String)
    func duplicate(rev: String)
    func merge(parents: [String])
    func squash(rev: String)
    func squash(rev: String, into: String)
    func absorb(rev: String)
    func revertChange(rev: String)
    func rebase(rev: String, dest: String)
    func abandon(rev: String)
    func compareWith(from: String, to: String)
    func diffBookmark(_ request: BookmarkDiffRequest)
    func showEvolog(rev: String)
}

protocol BookmarkActions: AnyObject {
    func createBookmark(name: String, rev: String)
    func deleteBookmark(name: String)
    func forgetBookmark(name: String)
    func moveBookmarkForward(name: String)
    func renameBookmark(oldName: String, newName: String)
    func trackBookmark(name: String, remote: String)
    func gitPush(bookmark: String)
    func gitFetch()
    func gitPullBookmark(name: String)
    func openPR(bookmark: String)
}
