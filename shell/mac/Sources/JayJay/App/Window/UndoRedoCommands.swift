import JayJayCore
import SwiftUI

struct UndoRedoCommands: Commands {
    private var router: EditUndoRouter {
        .shared
    }

    private var repo: RepositoryMenuHandler? {
        router.repoWindowIsKey ? ActiveRepoTracker.shared.handler : nil
    }

    var body: some Commands {
        CommandGroup(replacing: .undoRedo) {
            Button(undoTitle) { router.undo(repo: repo) }
                .keyboardShortcut("z", modifiers: .command)
                .disabled(!router.local.canUndo && repo?.operationSteps?.undoDescription == nil)
            Button(redoTitle) { router.redo(repo: repo) }
                .keyboardShortcut("z", modifiers: [.command, .shift])
                .disabled(!router.local.canRedo && repo?.operationSteps?.redoDescription == nil)
        }
    }

    private var undoTitle: String {
        if router.local.canUndo {
            return router.local.undoTitle
        }
        return repo?.operationSteps?.undoDescription.map { Localized.Menu.undoOperation(operation: $0) } ?? Localized.Repo.undo
    }

    private var redoTitle: String {
        if router.local.canRedo {
            return router.local.redoTitle
        }
        return repo?.operationSteps?.redoDescription.map { Localized.Menu.redoOperation(operation: $0) } ?? Localized.Menu.redo
    }
}
