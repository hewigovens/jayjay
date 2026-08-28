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
                case let .Review(message):
                    return unwrapCommandError(message)
                case let .Diff(message):
                    return unwrapCommandError(message)
                case let .DiffSelectionStale(path):
                    return "\(path): file changed since the diff was rendered — refresh and retry"
                case let .ConflictEditorStale(path):
                    return "\(path): conflict changed since the editor opened — refresh and retry"
                case let .FileEditorStale(path):
                    return "\(path): file changed since the editor opened — refresh and retry"
                case let .Internal(message):
                    return unwrapCommandError(message)
                case .Canceled:
                    return "Canceled"
            }
        }
        return localizedDescription
    }
}

/// Reduce a jj `command failed: … Error: … Caused by: …` failure to a plain sentence (drops wrapper/progress/--debug noise).
func unwrapCommandError(_ message: String) -> String {
    let stripped = message.replacingOccurrences(of: "command failed:", with: "")
    if let pushError = unwrapGitPushError(stripped) {
        return stripDebugHint(pushError)
    }
    var hints: [String] = []
    let parts = stripped.split(whereSeparator: \.isNewline).compactMap { line -> String? in
        let line = line.trimmingCharacters(in: .whitespaces)
        for marker in ["Error:", "Caused by:"] {
            if let range = line.range(of: marker) {
                return String(line[range.upperBound...]).trimmingCharacters(in: .whitespaces)
            }
        }
        if let range = line.range(of: "Hint:") {
            hints.append(String(line[range.upperBound...]).trimmingCharacters(in: .whitespaces))
        }
        return nil
    }
    let text = parts.isEmpty
        ? stripped.trimmingCharacters(in: .whitespacesAndNewlines)
        : parts.joined(separator: ": ")
    return hints.reduce(into: stripDebugHint(text)) { result, hint in
        guard !hint.isEmpty else { return }
        result += "\nHint: \(hint)"
    }
}

private func unwrapGitPushError(_ message: String) -> String? {
    let rawLines = message.split(whereSeparator: \.isNewline)
        .map { $0.trimmingCharacters(in: .whitespaces) }
    guard rawLines.contains(where: isGitPushFailureLine) else { return nil }

    var summary: String?
    var details: [String] = []
    for rawLine in rawLines {
        let line = stripCommandErrorPrefixes(rawLine)
        guard !line.isEmpty else { continue }
        if isGitPushSummary(line) {
            summary = line
            continue
        }
        if isGitPushProgressLine(line) {
            continue
        }
        details.append(line)
    }

    let uniqueDetails = details.reduce(into: [String]()) { result, line in
        if line != summary, !result.contains(line) {
            result.append(line)
        }
    }
    if let summary {
        return ([summary] + uniqueDetails).joined(separator: "\n")
    }
    return uniqueDetails.isEmpty ? nil : uniqueDetails.joined(separator: "\n")
}

private func isGitPushFailureLine(_ line: String) -> Bool {
    line.contains("git push failed") || line.contains("Failed to push some bookmarks")
}

private func isGitPushSummary(_ line: String) -> Bool {
    line.hasPrefix("Failed to push some bookmarks") || line.hasPrefix("Git process failed")
}

private func isGitPushProgressLine(_ line: String) -> Bool {
    line == "git:"
        || line.hasPrefix("Changes to push to ")
        || line.hasPrefix("bookmark:")
        || line.hasPrefix("Done importing changes")
}

private func stripCommandErrorPrefixes(_ line: String) -> String {
    var text = line
    var changed = true
    while changed {
        changed = false
        for prefix in ["git push failed:", "Error:", "Caused by:"] where text.hasPrefix(prefix) {
            text = String(text.dropFirst(prefix.count)).trimmingCharacters(in: .whitespaces)
            changed = true
        }
    }
    return text
}

private func stripDebugHint(_ message: String) -> String {
    var text = message
    if let hint = text.range(of: " (run with --debug") {
        text = String(text[..<hint.lowerBound])
    }
    return text
}
