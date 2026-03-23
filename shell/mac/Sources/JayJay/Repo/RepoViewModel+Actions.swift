import Foundation
import JayJayCore
#if canImport(FoundationModels)
    import FoundationModels
#endif

// MARK: - Action methods

extension RepoViewModel {
    func describeChange(rev: String, message: String) {
        describe(rev: rev, message: message)
    }

    func describe(rev: String, message: String) {
        perform(selecting: rev) { try $0.describe(rev: rev, message: message) }
    }

    func describeWorkingCopy(message: String) {
        perform { try $0.describe(rev: "@", message: message) }
    }

    func commit(message: String) {
        Task.detached { [repo] in
            do {
                try repo.commitWithSubmodules(message: message)
                await MainActor.run { [weak self] in
                    self?.reviewStore.clearAll()
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func generateCommitMessage() async -> String? {
        do {
            let summary = try repo.diffSummary()
            if summary.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                return nil
            }

            // 1. Try external AI CLIs (codex, then claude) via Rust
            let cliProvider = detectAiProvider()
            if !cliProvider.isEmpty {
                let cliResult: String? = await Task.detached { [repo] in
                    repo.generateCommitMessage(diffSummary: summary)
                }.value
                if let msg = cliResult, !msg.isEmpty {
                    await MainActor.run { [weak self] in self?.aiProvider = cliProvider }
                    return msg
                }
            }

            // 2. Fall back to Apple Foundation Models
            if let msg = await Self.generateWithLocalLLM(diffSummary: summary) {
                await MainActor.run { [weak self] in self?.aiProvider = "Apple Intelligence" }
                return msg
            }

            return nil
        } catch {
            await MainActor.run { [weak self] in
                self?.error = error.friendlyDescription
            }
            return nil
        }
    }

    func newChange(parent: String, message: String = "") {
        perform { try $0.newChange(parent: parent, message: message) }
    }

    func abandon(rev: String) {
        Task.detached { [repo] in
            do {
                try repo.abandon(rev: rev)
                await MainActor.run { [weak self] in
                    self?.selectedChangeId = nil
                    self?.selectedChange = nil
                    self?.refresh(selecting: "@")
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func squash(rev: String) {
        perform { try $0.squash(rev: rev, intoRev: nil) }
    }

    func squash(rev: String, into destination: String) {
        perform(selecting: destination) { try $0.squash(rev: rev, intoRev: destination) }
    }

    func edit(rev: String) {
        perform(selecting: rev) { try $0.edit(rev: rev) }
    }

    func graft(rev: String) {
        perform { try $0.graft(rev: rev) }
    }

    func absorb(rev: String) {
        perform { try $0.absorb(rev: rev) }
    }

    func backout(rev: String) {
        perform { try $0.backout(rev: rev) }
    }

    func merge(parents: [String]) {
        perform { try $0.merge(parentRevs: parents) }
    }

    func duplicate(rev: String) {
        perform { try $0.duplicate(rev: rev) }
    }

    func gitFetch() {
        Task.detached { [repo] in
            do {
                let msg = try repo.gitFetch(remote: "origin")
                await MainActor.run { [weak self] in
                    self?.info = msg
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func gitPush(bookmark: String = "") {
        Task.detached { [repo] in
            do {
                let msg = try repo.gitPush(bookmark: bookmark)
                await MainActor.run { [weak self] in
                    self?.info = msg
                    self?.refresh()
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func createBookmark(name: String, rev: String = "@") {
        perform(selecting: nil) { try $0.createBookmark(name: name, rev: rev) }
    }

    func moveBookmarkForward(name: String) {
        perform(selecting: nil) { try $0.moveBookmark(name: name, toRev: "@-") }
    }

    func deleteBookmark(name: String) {
        perform(selecting: nil) { try $0.deleteBookmark(name: name) }
    }

    func renameBookmark(oldName: String, newName: String) {
        perform(selecting: nil) { try $0.renameBookmark(oldName: oldName, newName: newName) }
    }

    func trackBookmark(name: String) {
        perform(selecting: nil) { try $0.trackBookmark(name: name, remote: "origin") }
    }

    func restoreFiles(rev: String, paths: [String]) {
        perform(selecting: rev) { try $0.restoreFiles(rev: rev, paths: paths) }
    }

    func deleteFiles(paths: [String]) {
        perform { try $0.deleteFiles(paths: paths) }
    }

    func ignoreAndUntrack(paths: [String]) {
        perform(selecting: nil) { try $0.ignoreAndUntrack(paths: paths) }
    }

    func split(rev: String, paths: [String], message: String = "", parallel: Bool = false) {
        perform { try $0.split(rev: rev, paths: paths, message: message, parallel: parallel) }
    }

    func moveToWorkingCopy(rev: String, paths: [String]) {
        perform { try $0.moveToWorkingCopy(rev: rev, paths: paths) }
    }

    func opLog() {
        Task.detached { [repo] in
            do {
                let entries = try repo.opLog()
                await MainActor.run { [weak self] in
                    self?.opLogEntries = entries
                }
            } catch {
                await MainActor.run { [weak self] in
                    self?.error = error.friendlyDescription
                }
            }
        }
    }

    func opRestore(opId: String) {
        perform { try $0.opRestore(opId: opId) }
    }

    @MainActor
    static func generateWithLocalLLM(diffSummary: String) async -> String? {
        #if canImport(FoundationModels)
            if #available(macOS 26.0, *) {
                return await generateWithFoundationModels(diffSummary: diffSummary)
            }
        #endif
        return nil
    }

    #if canImport(FoundationModels)
        @available(macOS 26.0, *)
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
