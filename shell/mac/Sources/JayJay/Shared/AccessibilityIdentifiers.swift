import Foundation

enum AID {
    enum Palette {
        static let textField = "commandPalette.searchField"

        static func item(_ title: String) -> String {
            "commandPalette.item.\(title)"
        }
    }

    enum Picker {
        static func row(_ id: String) -> String {
            "picker.row.\(id)"
        }
    }

    enum DAG {
        static func row(_ changeIdPrefix: String) -> String {
            "dag.row.\(changeIdPrefix)"
        }

        static func bookmark(_ name: String) -> String {
            "dag.bookmark.\(name)"
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
