import Foundation

protocol ChangeActions: AnyObject {
    func describeChange(rev: String, message: String)
    func restoreFiles(rev: String, paths: [String])
    func deleteFiles(paths: [String])
    func ignoreAndUntrack(paths: [String])
    func split(rev: String, paths: [String], message: String, parallel: Bool)
    func moveToWorkingCopy(rev: String, paths: [String])
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
    func backout(rev: String)
    func abandon(rev: String)
    func compareWith(from: String, to: String)
}

protocol BookmarkActions: AnyObject {
    func createBookmark(name: String, rev: String)
    func deleteBookmark(name: String)
    func moveBookmarkForward(name: String)
    func renameBookmark(oldName: String, newName: String)
    func trackBookmark(name: String)
    func gitPush(bookmark: String)
    func gitFetch()
}
