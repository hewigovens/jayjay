import Foundation
import JayJayCore

enum ReviewCommand {
    case notes(repo: String, format: String, includeResolved: Bool)
    case resolveNote(id: String, repo: String)
    case addNote(repo: String, file: String, line: UInt32, side: String, message: String)

    init(arguments: [String]) throws {
        guard let subcommand = arguments.first else {
            throw CommandLineUsageError.usage("missing review subcommand")
        }
        let parser = CommandLineArgumentParser(Array(arguments.dropFirst()))
        switch subcommand {
            case "notes":
                self = try Self.parseNotes(parser)
            case "resolve-note":
                self = try Self.parseResolveNote(parser)
            case "add-note":
                self = try Self.parseAddNote(parser)
            default:
                throw CommandLineUsageError.usage("unknown review subcommand: \(subcommand)")
        }
    }

    func run() throws -> String {
        switch self {
            case let .notes(repo, format, includeResolved):
                try reviewNotesOutput(repoPath: repo, format: format, includeResolved: includeResolved)
            case let .resolveNote(id, repo):
                try resolveReviewNote(repoPath: repo, id: id)
            case let .addNote(repo, file, line, side, message):
                try addReviewNote(
                    repoPath: repo,
                    file: file,
                    line: line,
                    side: side,
                    message: message
                )
        }
    }

    private static func parseNotes(_ parser: CommandLineArgumentParser) throws -> Self {
        let repo = try parser.option("--repo") ?? "."
        let format = try parser.option("--format") ?? "text"
        guard format == "text" || format == "json" else {
            throw CommandLineUsageError.usage("unsupported review notes format: \(format)")
        }
        let includeResolved = parser.flag("--include-resolved")
        try parser.finish()
        return .notes(repo: repo, format: format, includeResolved: includeResolved)
    }

    private static func parseResolveNote(_ parser: CommandLineArgumentParser) throws -> Self {
        let repo = try parser.option("--repo") ?? "."
        guard let id = parser.positional() else {
            throw CommandLineUsageError.usage("missing review note id")
        }
        try parser.finish()
        return .resolveNote(id: id, repo: repo)
    }

    private static func parseAddNote(_ parser: CommandLineArgumentParser) throws -> Self {
        let repo = try parser.option("--repo") ?? "."
        guard let file = try parser.option("--file") else {
            throw CommandLineUsageError.usage("missing --file")
        }
        guard let lineValue = try parser.option("--line"), let line = UInt32(lineValue) else {
            throw CommandLineUsageError.usage("missing or invalid --line")
        }
        let side = try parser.option("--side") ?? "new"
        guard side == "new" || side == "old" else {
            throw CommandLineUsageError.usage("unsupported review note side: \(side)")
        }
        guard let message = try parser.option("--message", alias: "-m") else {
            throw CommandLineUsageError.usage("missing --message")
        }
        try parser.finish()
        return .addNote(repo: repo, file: file, line: line, side: side, message: message)
    }
}

private final class CommandLineArgumentParser {
    private let arguments: [String]
    private var consumed: Set<Int> = []

    init(_ arguments: [String]) {
        self.arguments = arguments
    }

    func flag(_ name: String) -> Bool {
        guard let index = firstUnconsumedIndex(of: name) else { return false }
        consumed.insert(index)
        return true
    }

    func option(_ name: String, alias: String? = nil) throws -> String? {
        for index in arguments.indices where !consumed.contains(index) {
            let arg = arguments[index]
            if arg == name || arg == alias {
                let valueIndex = index + 1
                guard valueIndex < arguments.count, !arguments[valueIndex].hasPrefix("-") else {
                    throw CommandLineUsageError.usage("missing value for \(arg)")
                }
                consumed.insert(index)
                consumed.insert(valueIndex)
                return arguments[valueIndex]
            }
            if arg.hasPrefix("\(name)=") {
                consumed.insert(index)
                return String(arg.dropFirst(name.count + 1))
            }
        }
        return nil
    }

    func positional() -> String? {
        for index in arguments.indices where !consumed.contains(index) {
            let arg = arguments[index]
            guard !arg.hasPrefix("-") else { continue }
            consumed.insert(index)
            return arg
        }
        return nil
    }

    func finish() throws {
        let extras = arguments.indices.filter { !consumed.contains($0) }
        guard extras.isEmpty else {
            throw CommandLineUsageError.usage("unexpected argument: \(arguments[extras[0]])")
        }
    }

    private func firstUnconsumedIndex(of value: String) -> Int? {
        arguments.indices.first { !consumed.contains($0) && arguments[$0] == value }
    }
}

private enum CommandLineUsageError: LocalizedError {
    case usage(String)

    var errorDescription: String? {
        switch self {
            case let .usage(message): message
        }
    }
}
