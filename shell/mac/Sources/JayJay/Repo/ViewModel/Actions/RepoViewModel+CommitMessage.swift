import Foundation
import JayJayCore
#if canImport(FoundationModels)
    import FoundationModels
#endif

extension RepoViewModel {
    func generateCommitMessage() async -> String? {
        do {
            let summary = try repo.diffSummary()
            if summary.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                return nil
            }

            let cliProvider = detectAiProvider()
            if !cliProvider.isEmpty {
                let cliResult: String? = await Task.detached { [repo] in
                    repo.generateCommitMessage(diffSummary: summary)
                }.value
                if let message = cliResult, !message.isEmpty {
                    await MainActor.run { [weak self] in self?.aiProvider = cliProvider }
                    return message
                }
            }

            if let message = await Self.generateWithLocalLLM(diffSummary: summary) {
                await MainActor.run { [weak self] in self?.aiProvider = "Apple Intelligence" }
                return message
            }

            return nil
        } catch {
            await MainActor.run { [weak self] in
                self?.present(error: error)
            }
            return nil
        }
    }

    @MainActor
    static func generateWithLocalLLM(diffSummary: String) async -> String? {
        #if canImport(FoundationModels)
            return await generateWithFoundationModels(diffSummary: diffSummary)
        #else
            return nil
        #endif
    }

    #if canImport(FoundationModels)
        @MainActor
        private static func generateWithFoundationModels(diffSummary: String) async -> String? {
            do {
                let session = FoundationModels.LanguageModelSession()
                let prompt = """
                \(commitMessagePrompt())
                Changed files:

                \(diffSummary)
                """
                let response = try await session.respond(to: prompt)
                let text = response.content.trimmingCharacters(in: .whitespacesAndNewlines)
                return text.isEmpty ? nil : text
            } catch {
                return nil
            }
        }
    #endif
}
