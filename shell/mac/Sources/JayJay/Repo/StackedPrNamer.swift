import Foundation
#if canImport(FoundationModels)
    import FoundationModels
#endif

/// On-device branch-name generation for stacked PRs. Returns `nil` when Apple
/// Intelligence is unavailable or generation fails, so callers fall back to the
/// slug-based name that core already proposed.
enum StackedPrNamer {
    /// Whether on-device generation is available (gates the "Generate bookmarks" button).
    static var isAvailable: Bool {
        #if canImport(FoundationModels)
            return true
        #else
            return false
        #endif
    }

    @MainActor
    static func branchName(from description: String) async -> String? {
        let trimmed = description.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }
        #if canImport(FoundationModels)
            return await generate(from: trimmed)
        #else
            return nil
        #endif
    }

    #if canImport(FoundationModels)
        @MainActor
        private static func generate(from description: String) async -> String? {
            do {
                let session = FoundationModels.LanguageModelSession()
                let prompt = """
                Generate a concise git branch name in kebab-case (lowercase words \
                separated by hyphens, no spaces or punctuation, at most \(maxWords) \
                words) that summarizes this change. Output only the branch name.

                \(description)
                """
                let response = try await session.respond(to: prompt)
                return slug(response.content)
            } catch {
                return nil
            }
        }
    #endif

    /// Cap the generated slug to match core's `MAX_SLUG_WORDS`.
    private static let maxWords = 5

    /// Sanitize a model reply into a safe branch slug — the model may wrap it in
    /// quotes or markdown, or return more than the requested number of words.
    static func slug(_ raw: String) -> String? {
        var words: [String] = []
        var word = ""
        for ch in raw {
            if ch.isASCII, ch.isLetter || ch.isNumber {
                word.append(Character(ch.lowercased()))
            } else if !word.isEmpty {
                words.append(word)
                word = ""
                if words.count == maxWords { break }
            }
        }
        if !word.isEmpty, words.count < maxWords { words.append(word) }
        return words.isEmpty ? nil : words.joined(separator: "-")
    }
}
