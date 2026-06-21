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
        static func row(_ path: String) -> String {
            "file.row.\(path)"
        }
    }

    enum Diff {
        static let section = "diff.section"
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
        static let commit = "commitBox.commit"
    }

    enum Conflict {
        static func useOurs(_ path: String) -> String {
            "conflict.useOurs.\(path)"
        }

        static func useTheirs(_ path: String) -> String {
            "conflict.useTheirs.\(path)"
        }
    }
}
