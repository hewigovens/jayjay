import Foundation

enum AID {
    enum Palette {
        static let textField = "commandPalette.searchField"

        static func item(_ title: String) -> String {
            "commandPalette.item.\(title)"
        }
    }

    enum DAG {
        static func row(_ changeIdPrefix: String) -> String {
            "dag.row.\(changeIdPrefix)"
        }
    }

    enum FileList {
        static let showInFinder = "file.context.showInFinder"

        static func row(_ path: String) -> String {
            "file.row.\(path)"
        }
    }

    enum Diff {
        static let section = "diff.section"
        static let gutter = "diff.gutter"
        static let text = "diff.text"
    }

    enum ReviewNote {
        static let body = "reviewNote.body"
        static let contextCode = "reviewNote.contextCode"

        static func activeCount(_ count: Int) -> String {
            "reviewNote.activeCount.\(count)"
        }

        static func fileCount(path: String, count: Int) -> String {
            "reviewNote.fileCount.\(count).\(path)"
        }
    }

    enum Detail {
        /// Counts are encoded in the id so UI tests assert on existence, not a11y value.
        static func diffStats(insertions: UInt32, deletions: UInt32) -> String {
            "detail.diffStats.\(insertions).\(deletions)"
        }
    }

    enum Compare {
        static let banner = "compare.banner"
    }

    enum CommitBox {
        static let summary = "commitBox.summary"
        static let draft = "commitBox.draft"
        static let save = "commitBox.save"
        static let commit = "commitBox.commit"
    }

    enum SplitSheet {
        static let openButton = "splitSheet.open"
        static let messageField = "splitSheet.message"

        static func fileRow(_ path: String) -> String {
            "splitSheet.file.\(path)"
        }
    }

    enum Conflict {
        static func useOurs(_ path: String) -> String {
            "conflict.useOurs.\(path)"
        }

        static func useTheirs(_ path: String) -> String {
            "conflict.useTheirs.\(path)"
        }

        static func resolveInJayJay(_ path: String) -> String {
            "conflict.resolveInJayJay.\(path)"
        }

        static let editorResult = "conflict.editor.result"
        static let editorPreparing = "conflict.editor.preparing"
        static let editorModal = "conflict.editor.modal"
        static let editorSave = "conflict.editor.save"
        static let editorCancel = "conflict.editor.cancel"
        static let editorHunks = "conflict.editor.hunks"
        static let editorRaw = "conflict.editor.raw"
        static let editorHunkList = "conflict.editor.hunkList"

        static func hunkUse(_ index: UInt32, _ source: String) -> String {
            "conflict.editor.hunk.\(index).use.\(source)"
        }
    }

    enum FileEditor {
        static let modal = "fileEditor.modal"
        static let content = "fileEditor.content"
        static let preparing = "fileEditor.preparing"
        static let save = "fileEditor.save"
        static let cancel = "fileEditor.cancel"

        static func open(_ path: String) -> String {
            "fileEditor.open.\(path)"
        }
    }

    enum Settings {
        static let copyJJToolConfig = "settings.copyJJToolConfig"
    }

    enum ExternalTool {
        static let diff = "externalTool.diff"
        static let merge = "externalTool.merge"
        static let baseVisibility = "externalTool.baseVisibility"
        static let save = "externalTool.save"

        static func fileToggle(_ path: String) -> String {
            "externalTool.fileToggle.\(path)"
        }

        static func useSource(_ source: String) -> String {
            "externalTool.useSource.\(source)"
        }
    }

    enum DiffEdit {
        static let open = "diffEdit.open"
        static let expandAll = "diffEdit.expandAll"
        static let collapseAll = "diffEdit.collapseAll"
        static let cancel = "diffEdit.cancel"

        static func fileToggle(_ path: String) -> String {
            "diffEdit.fileToggle.\(path)"
        }
    }
}
