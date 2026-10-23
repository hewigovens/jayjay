import JayJayCore

protocol RevsetActions: AnyObject {
    var revsetFilter: RevsetFilterState { get }
    var defaultRevsetPreset: RevsetPreset { get }
    func revsetError(_ revset: String) -> String?
    func applyFilter(_ revset: String, selecting revision: String)
    func returnToPreviousRevset()
    func forgetRecentRevset(_ revset: String)
}

extension RevsetActions {
    func applyFilter(_ revset: String) {
        applyFilter(revset, selecting: "@")
    }

    func applyTyped(_ text: String, bookmarks: [BookmarkInfo]) -> String? {
        let revset = typedRevset(text: text, bookmarks: bookmarks, defaultRevset: defaultRevsetPreset)
        if revset == text, let error = revsetError(text) {
            return error
        }
        applyFilter(revset)
        return nil
    }
}
