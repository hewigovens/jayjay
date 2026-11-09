import AppKit
import JayJayCore
import SwiftUI

enum AppShortcut: String, Decodable {
    case openRepository = "app.open_repository"
    case commandPalette = "repo.command_palette"
    case refresh = "repo.refresh"
    case keyboardShortcuts = "app.keyboard_shortcuts"
    case settings = "app.settings"
    case toggleSidebar = "repo.toggle_sidebar"
    case zoomIn = "app.zoom_in"
    case zoomOut = "app.zoom_out"
    case resetZoom = "app.reset_zoom"
    case revsetFilter = "repo.filter_by_revset"
    case bookmarkManager = "repo.bookmark_manager"
    case repoOverview = "repo.overview"
    case filterOverviewLanes = "overview.filter_lanes"
    case operationLog = "repo.undo_last_operation"
    case showInFinder = "repo.show_in_file_manager"
    case findInDiff = "repo.find_in_diff"
    case saveEditedFile = "text.save_file"
    case saveReviewNote = "text.save_review_note"
    case expandAllFiles = "diff_editor.expand_all_files"
    case collapseAllFiles = "diff_editor.collapse_all_files"
    case acceptLeftHunk = "merge.accept_left"
    case acceptRightHunk = "merge.accept_right"
    case previousHunk = "merge.previous_conflict"
    case nextHunk = "merge.next_conflict"

    case fetch = "repo.fetch"
    case push = "repo.push"
    case newWorkspace = "repo.new_workspace"
    case importPullRequest = "repo.import_pull_request"
    case rebaseOntoTrunk = "repo.rebase_onto_trunk"
    case bisect = "repo.bisect"
    case cleanUpStaleBookmarks = "repo.clean_up_stale_bookmarks"
    case toggleSideBySide = "repo.toggle_side_by_side"
    case toggleIgnoreWhitespace = "repo.toggle_ignore_whitespace"
    case toggleTreeFileList = "repo.toggle_tree_file_list"
    case newChange = "repo.new_change"
    case editChange = "repo.edit_change"
    case describeChange = "repo.describe_change"
    case squashChange = "repo.squash_change"
    case newBookmark = "repo.new_bookmark"
    case showEvolution = "repo.show_evolution"
    case abandon = "changes.abandon"

    var symbol: String {
        ShortcutStore.shared.byID[rawValue]?.primary?.caps.joined() ?? ""
    }

    func help(_ title: String) -> String {
        symbol.isEmpty ? title : Localized.Common.withShortcut(title: title, shortcut: symbol)
    }

    func matches(_ event: NSEvent) -> Bool {
        guard let binding = ShortcutStore.shared.byID[rawValue] else { return false }
        return binding.effectiveChords.contains { $0.text == event.shortcutText }
    }
}

extension View {
    func keyboardShortcut(_ shortcut: AppShortcut) -> some View {
        keyboardShortcut(ShortcutStore.shared.byID[shortcut.rawValue]?.primary?.keyboardShortcut)
    }
}
