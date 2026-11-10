import AppKit
import Observation

@MainActor
@Observable
final class EditUndoRouter {
    static let shared = EditUndoRouter()

    struct LocalHistory: Equatable {
        var canUndo = false
        var canRedo = false
        var undoTitle = ""
        var redoTitle = ""

        init() {}

        @MainActor
        init(window: NSWindow?) {
            guard let manager = LocalHistory.manager(in: window) else { return }
            canUndo = manager.canUndo
            canRedo = manager.canRedo
            undoTitle = manager.undoMenuItemTitle
            redoTitle = manager.redoMenuItemTitle
        }

        @MainActor
        static func manager(in window: NSWindow?) -> UndoManager? {
            let responder = window?.firstResponder
            if EditUndoRouter.isTextInput(responder) {
                return responder?.undoManager
            }
            return EditUndoRouter.acceptsRepoSteps(window) ? nil : responder?.undoManager ?? window?.undoManager
        }
    }

    private(set) var local = LocalHistory()
    private(set) var repoWindowIsKey = false

    @ObservationIgnored private var firstResponderObservation: NSKeyValueObservation?

    private init() {
        let center = NotificationCenter.default
        let names: [Notification.Name] = [
            NSWindow.didBecomeKeyNotification, NSWindow.didResignKeyNotification,
            NSWindow.willBeginSheetNotification, NSWindow.didEndSheetNotification,
            .NSUndoManagerDidCloseUndoGroup, .NSUndoManagerDidUndoChange, .NSUndoManagerDidRedoChange,
            NSMenu.didBeginTrackingNotification
        ]
        for name in names {
            center.addObserver(forName: name, object: nil, queue: .main) { [weak self] notification in
                MainActor.assumeIsolated {
                    if notification.name == NSWindow.didBecomeKeyNotification {
                        self?.observeFirstResponder(of: notification.object as? NSWindow)
                    }
                    self?.update()
                }
            }
        }
        observeFirstResponder(of: NSApp.keyWindow)
        update()
    }

    func undo(repo: RepositoryMenuHandler?) {
        let window = NSApp.keyWindow
        if LocalHistory(window: window).canUndo {
            LocalHistory.manager(in: window)?.undo()
        } else if Self.acceptsRepoSteps(window) {
            repo?.undoOperation()
        }
    }

    func redo(repo: RepositoryMenuHandler?) {
        let window = NSApp.keyWindow
        if LocalHistory(window: window).canRedo {
            LocalHistory.manager(in: window)?.redo()
        } else if Self.acceptsRepoSteps(window) {
            repo?.redoOperation()
        }
    }

    static func acceptsRepoSteps(_ window: NSWindow?) -> Bool {
        guard let window else { return false }
        return window.representedURL != nil && window.attachedSheet == nil && window.sheetParent == nil && !isTextInput(window.firstResponder)
    }

    /// Read-only text (the description, diff panes) takes focus for selection but has nothing of its own to undo.
    private static func isTextInput(_ responder: NSResponder?) -> Bool {
        (responder as? NSText)?.isEditable == true
    }

    private func observeFirstResponder(of window: NSWindow?) {
        firstResponderObservation = window?.observe(\.firstResponder) { [weak self] _, _ in
            MainActor.assumeIsolated { self?.update() }
        }
    }

    private func update() {
        let window = NSApp.keyWindow
        let local = LocalHistory(window: window)
        if local != self.local {
            self.local = local
        }
        let repoWindowIsKey = Self.acceptsRepoSteps(window)
        if repoWindowIsKey != self.repoWindowIsKey {
            self.repoWindowIsKey = repoWindowIsKey
        }
    }
}
