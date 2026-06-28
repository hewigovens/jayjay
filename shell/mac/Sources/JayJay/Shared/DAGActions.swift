import Foundation

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
