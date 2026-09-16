import FoundationModels
import JayJayCore

@Generable
struct GeneratedCommitMessage {
    @Guide(
        description: "One line under 72 characters, starting with one of Add, Update, Fix, Refactor, Remove, Docs, Test or Chore, then a colon and what changed."
    )
    var summary: String

    @Guide(description: "What changed and why, one short sentence each, no leading dash.", .count(1 ... 4))
    var bullets: [String]

    var message: String {
        let body = bullets
            .map { "- \($0.trimmingCharacters(in: .whitespacesAndNewlines))" }
            .joined(separator: "\n")
        return joinCommitMessage(summary: summary, body: body)
    }
}

/// The on-device model rejects a `.pattern` guide, so `branchNameSlug` still gates the reply.
@Generable
private struct GeneratedBranchName {
    @Guide(
        description: "Lowercase kebab-case summary of the change: two to five words joined by hyphens, no spaces and no punctuation."
    )
    var name: String
}

enum AppleIntelligence {
    static var isAvailable: Bool {
        SystemLanguageModel.default.availability == .available
    }

    static func commitMessage(for excerpt: DiffExcerpt) async -> String? {
        guard let prompt = await prompt(for: excerpt) else { return nil }
        return await respond(
            to: prompt,
            instructions: commitMessageInstructions,
            generating: GeneratedCommitMessage.self
        )?.message
    }

    static func branchName(from description: String) async -> String? {
        guard let generated = await respond(
            to: description,
            instructions: branchNameInstructions,
            generating: GeneratedBranchName.self
        ) else { return nil }
        let slug = branchNameSlug(text: generated.name)
        return slug.isEmpty ? nil : slug
    }

    private static let commitMessageInstructions =
        "Summarize a source-control change for a commit message. Describe what the diff does, not how it is formatted."

    private static let branchNameInstructions = "Name a git branch after a source-control change."

    /// Without a cap, a runaway summary can fill the context and take over a minute.
    private static let replyTokenLimit = 512

    /// Prompt processing dominates latency: about 7 s at this size, 13 to 18 s for the full 8K window.
    private static let promptTokenLimit = 3500

    private static let maxBytesPerToken = 4

    private static func prompt(for excerpt: DiffExcerpt) async -> String? {
        guard #available(macOS 26.4, *) else {
            return diffExcerptText(excerpt: excerpt, maxBytes: nil)
        }
        return await fittedPrompt(for: excerpt)
    }

    /// Bisects the byte cap of the whole excerpt, stat included, so a long stat shrinks instead of starving the diff.
    @available(macOS 26.4, *)
    private static func fittedPrompt(for excerpt: DiffExcerpt) async -> String? {
        let model = SystemLanguageModel.default
        let budget = min(promptTokenLimit, model.contextSize / 2)
        var fitting: String?
        var low = 0
        var high = budget * maxBytesPerToken
        var bytes = high
        while true {
            let prompt = diffExcerptText(excerpt: excerpt, maxBytes: UInt32(bytes))
            guard let tokens = try? await model.tokenCount(for: prompt) else { return nil }
            if tokens <= budget {
                fitting = prompt
                low = bytes
            } else {
                high = bytes
            }
            if high - low <= 256 {
                return fitting
            }
            bytes = (low + high) / 2
        }
    }

    private static func respond<Content: Generable>(
        to prompt: String,
        instructions: String,
        generating _: Content.Type
    ) async -> Content? {
        guard isAvailable else { return nil }
        let session = LanguageModelSession(instructions: instructions)
        let options = GenerationOptions(maximumResponseTokens: replyTokenLimit)
        return try? await session.respond(to: prompt, generating: Content.self, options: options).content
    }
}
