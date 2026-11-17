import AppKit
import JayJayCore

@objc(OpenRepositoryScriptCommand)
final class OpenRepositoryScriptCommand: NSScriptCommand {
    override func performDefaultImplementation() -> Any? {
        MainActor.assumeIsolated {
            guard let path = directParameter as? String, path.hasPrefix("/") else {
                ScriptError(number: -1700, message: "Expected an absolute local repository path.").report(to: self)
                return nil
            }
            guard let manager = ScriptingHost.windowManager,
                  manager.settings.hasCompletedOnboarding,
                  manager.openWindowAction != nil
            else {
                ScriptError(number: -1708, message: "Complete onboarding and wait for JayJay to finish launching.").report(to: self)
                return nil
            }
            complete {
                let root: String
                do {
                    root = try await Task.detached {
                        let root = enclosingWorkspaceRoot(path: path) ?? path
                        _ = try JayJayRepo.open(path: root)
                        return root
                    }.value
                } catch {
                    throw ScriptError(number: -1728, message: "Cannot open repository at \(path): \(error.friendlyDescription)")
                }
                guard !manager.isRemovingRepo(at: root) else {
                    throw ScriptError(number: -1728, message: "The repository is being removed: \(path)")
                }
                let wasLoaded = manager.registeredViewModel(at: root)?.hasFinishedFirstLoad == true
                manager.openRepo(root)
                guard let model = await manager.waitForRegistration(at: root, timeout: .seconds(30)) else {
                    guard !manager.isRemovingRepo(at: root) else {
                        throw ScriptError(number: -1728, message: "The repository is being removed: \(path)")
                    }
                    throw ScriptError(number: -1712, message: "The repository window did not appear within 30 seconds.")
                }
                await model.waitForFirstLoad()
                try ScriptRepository.requireLoaded(model, reportingError: !wasLoaded)
                return ScriptRepository(viewModel: model)
            }
            return nil
        }
    }
}
