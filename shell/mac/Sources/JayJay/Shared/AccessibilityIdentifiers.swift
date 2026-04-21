import Foundation

enum AID {
    enum Palette {
        static let textField = "commandPalette.searchField"
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

    enum Conflict {
        static func useOurs(_ path: String) -> String {
            "conflict.useOurs.\(path)"
        }

        static func useTheirs(_ path: String) -> String {
            "conflict.useTheirs.\(path)"
        }
    }
}
