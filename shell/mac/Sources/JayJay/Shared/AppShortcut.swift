import AppKit
import SwiftUI

enum AppShortcut: CaseIterable {
    case openRepository
    case commandPalette
    case refresh
    case keyboardShortcuts
    case settings
    case toggleSidebar
    case zoomIn
    case zoomOut
    case resetZoom
    case revsetFilter
    case bookmarkManager
    case repoOverview
    case filterOverviewLanes
    case undoLastOperation
    case showInFinder
    case findInDiff
    case saveEditedFile
    case saveReviewNote
    case expandAllFiles
    case collapseAllFiles
    case acceptLeftHunk
    case acceptRightHunk
    case previousHunk
    case nextHunk

    var key: KeyEquivalent {
        switch self {
            case .openRepository, .repoOverview: "o"
            case .commandPalette: "p"
            case .refresh: "r"
            case .keyboardShortcuts: "/"
            case .settings: ","
            case .toggleSidebar, .saveEditedFile: "s"
            case .zoomIn: "+"
            case .zoomOut: "-"
            case .resetZoom: "0"
            case .revsetFilter: "l"
            case .bookmarkManager: "b"
            case .filterOverviewLanes, .findInDiff, .showInFinder: "f"
            case .undoLastOperation: "u"
            case .saveReviewNote: .return
            case .expandAllFiles: "e"
            case .collapseAllFiles: "c"
            case .acceptLeftHunk: .leftArrow
            case .acceptRightHunk: .rightArrow
            case .previousHunk: .upArrow
            case .nextHunk: .downArrow
        }
    }

    var modifiers: EventModifiers {
        switch self {
            case .commandPalette, .bookmarkManager, .repoOverview, .undoLastOperation: [.command, .shift]
            case .toggleSidebar: [.control, .command]
            case .showInFinder, .expandAllFiles, .collapseAllFiles: [.option, .command]
            case .acceptLeftHunk, .acceptRightHunk, .previousHunk, .nextHunk: .option
            default: .command
        }
    }

    var keyCaps: [String] {
        Self.modifierKeys.filter { modifiers.contains($0.modifier) }.map(\.glyph) + [keyGlyph]
    }

    var symbol: String {
        keyCaps.joined()
    }

    func matches(_ event: NSEvent) -> Bool {
        Self.modifierKeys.allSatisfy { modifiers.contains($0.modifier) == event.modifierFlags.contains($0.flag) }
            && event.charactersIgnoringModifiers?.lowercased() == String(key.character)
    }

    private struct ModifierKey {
        let modifier: EventModifiers
        let flag: NSEvent.ModifierFlags
        let glyph: String
    }

    private static let modifierKeys = [
        ModifierKey(modifier: .control, flag: .control, glyph: "⌃"),
        ModifierKey(modifier: .option, flag: .option, glyph: "⌥"),
        ModifierKey(modifier: .shift, flag: .shift, glyph: "⇧"),
        ModifierKey(modifier: .command, flag: .command, glyph: "⌘")
    ]

    private var keyGlyph: String {
        switch key {
            case .return: "↩"
            case .leftArrow: "←"
            case .rightArrow: "→"
            case .upArrow: "↑"
            case .downArrow: "↓"
            case "-": "−"
            default: String(key.character).uppercased()
        }
    }
}

extension View {
    func keyboardShortcut(_ shortcut: AppShortcut) -> some View {
        keyboardShortcut(shortcut.key, modifiers: shortcut.modifiers)
    }
}
