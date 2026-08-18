import Foundation

let trunkBookmarkNames: Set<String> = ["main", "master", "trunk"]

/// Matches bare "main" as well as remote-qualified forms like "main@origin".
func isTrunkBookmark(_ name: String) -> Bool {
    let bare = name.split(separator: "@").first.map(String.init) ?? name
    return trunkBookmarkNames.contains(bare)
}

/// DAG chips may drop a conflicted target even on trunk; whole-bookmark delete stays hidden for resolved trunk names.
func canRemoveBookmarkFromChip(_ name: String, conflicted: Bool) -> Bool {
    conflicted || !isTrunkBookmark(name)
}
