import JayJayCore

extension AiProvider {
    /// Blocks, so call it off the main actor.
    var isReady: Bool {
        self == .appleIntelligence ? AppleIntelligence.isAvailable : aiProviderIsInstalled(provider: self)
    }

    func commitMessage(for excerpt: DiffExcerpt) async -> String? {
        if self == .appleIntelligence {
            return await AppleIntelligence.commitMessage(for: excerpt)
        }
        return await Task.detached { generateCommitMessage(provider: self, excerpt: excerpt) }.value
    }

    func branchName(from description: String) async -> String? {
        if self == .appleIntelligence {
            return await AppleIntelligence.branchName(from: description)
        }
        return await Task.detached { generateBranchName(provider: self, description: description) }.value
    }
}

extension [AiProvider] {
    func firstReadyLabel() async -> String {
        await Task.detached { [self] in first(where: \.isReady)?.label ?? "" }.value
    }

    func firstAnswer(_ ask: (AiProvider) async -> String?) async -> (provider: AiProvider, text: String)? {
        for provider in self {
            if let text = await ask(provider) {
                return (provider, text)
            }
        }
        return nil
    }
}
