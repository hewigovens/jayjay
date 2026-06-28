import Foundation

let trunkBookmarkNames: Set<String> = ["main", "master", "trunk"]

/// Matches bare "main" as well as remote-qualified forms like "main@origin".
func isTrunkBookmark(_ name: String) -> Bool {
    let bare = name.split(separator: "@").first.map(String.init) ?? name
    return trunkBookmarkNames.contains(bare)
}
