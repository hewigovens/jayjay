import Foundation
import JayJayCore

extension Error {
    var friendlyDescription: String {
        if let jjError = self as? JayJayError {
            switch jjError {
                case let .RepoNotFound(path):
                    return "No Jujutsu repository found at \(URL(fileURLWithPath: path).lastPathComponent)"
                case let .RevNotFound(rev):
                    return "Revision not found: \(rev)"
                case let .Internal(message):
                    return unwrapCommandError(message)
            }
        }
        return localizedDescription
    }
}

/// Reduce a jj `command failed: … Error: … Caused by: …` failure to a plain sentence (drops wrapper/progress/--debug noise).
func unwrapCommandError(_ message: String) -> String {
    let stripped = message.replacingOccurrences(of: "command failed:", with: "")
    let parts = stripped.split(whereSeparator: \.isNewline).compactMap { line -> String? in
        let line = line.trimmingCharacters(in: .whitespaces)
        for prefix in ["Error:", "Caused by:"] where line.hasPrefix(prefix) {
            return String(line.dropFirst(prefix.count)).trimmingCharacters(in: .whitespaces)
        }
        return nil
    }
    var text = parts.isEmpty
        ? stripped.trimmingCharacters(in: .whitespacesAndNewlines)
        : parts.joined(separator: ": ")
    if let hint = text.range(of: " (run with --debug") {
        text = String(text[..<hint.lowerBound])
    }
    return text
}
