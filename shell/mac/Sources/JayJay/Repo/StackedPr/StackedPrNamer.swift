import Foundation
import JayJayCore

enum StackedPrNamer {
    static func branchName(from description: String, using order: [AiProvider]) async -> String? {
        let trimmed = description.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }
        return await order.firstAnswer { await $0.branchName(from: trimmed) }?.text
    }
}
