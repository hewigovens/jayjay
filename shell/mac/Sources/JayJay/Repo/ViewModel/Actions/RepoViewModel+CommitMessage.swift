import Foundation
import JayJayCore
#if canImport(FoundationModels)
    import FoundationModels
#endif

extension RepoViewModel {
    @MainActor
    func generateCommitMessage() async -> String? {
        do {
            let summary = try await awaitRepoTask { try $0.diffSummary() }
            if summary.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                return nil
            }

            let cliProvider = detectAiProvider()
            if !cliProvider.isEmpty {
                let cliResult: String? = try await awaitRepoTask {
                    $0.generateCommitMessage(diffSummary: summary)
                }
                if let message = cliResult, !message.isEmpty {
                    aiProvider = cliProvider
                    return message
                }
            }

            if let message = await Self.generateWithLocalLLM(diffSummary: summary) {
                aiProvider = "Apple Intelligence"
                return message
            }

            return nil
        } catch {
            present(error: error)
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
