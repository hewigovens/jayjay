import Foundation

protocol BookmarkActions: AnyObject {
    func createBookmark(name: String, rev: String)
    func deleteBookmark(name: String)
    func removeBookmark(name: String, fromRev: String)
    func forgetBookmark(name: String)
    func moveBookmarkForward(name: String)
    func renameBookmark(oldName: String, newName: String)
    func trackBookmark(name: String, remote: String)
    func gitPush(bookmark: String)
    func gitFetch()
    func gitPullBookmark(name: String)
    func openPR(bookmark: String)
}
