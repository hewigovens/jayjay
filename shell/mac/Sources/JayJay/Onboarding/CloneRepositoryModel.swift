import Foundation
import JayJayCore

@MainActor
@Observable
final class CloneRepositoryModel {
    var source = "" {
        didSet {
            if nameFollowsSource {
                name = cloneDirectoryName(source: source)
            }
        }
    }

    /// Typing a name detaches it from the URL; clearing it or matching the URL's name reattaches it.
    var name = "" {
        didSet {
            nameFollowsSource = name.isEmpty || name == cloneDirectoryName(source: source)
            guard !syncingLocation else { return }
            updateLocation()
        }
    }

    /// The clone's own folder; its last component is the name and the rest is the folder.
    var location: String {
        didSet {
            guard !syncingLocation else { return }
            let parts = splitCloneLocation(location: location)
            syncLocation {
                folder = parts.folder
                name = parts.name
            }
        }
    }

    private(set) var folder: String
    private(set) var isCloning = false
    private(set) var errorMessage: String?
    private var nameFollowsSource = true
    private var syncingLocation = false
    private var token: JayJaySyncToken?
    private var finished: DispatchSemaphore?

    init(recentRepositories: [String]) {
        let folder = (defaultCloneLocation(recentRepositories: recentRepositories) as NSString).abbreviatingWithTildeInPath
        self.folder = folder
        location = folder
    }

    var canClone: Bool {
        !isCloning && !source.trimmingCharacters(in: .whitespaces).isEmpty
            && !name.trimmingCharacters(in: .whitespaces).isEmpty
    }

    func choose(folder path: String) {
        folder = (path as NSString).abbreviatingWithTildeInPath
        updateLocation()
    }

    func clone(onCloned: @escaping (String) -> Void) {
        guard canClone else { return }
        let token = JayJaySyncToken.standalone()
        let finished = DispatchSemaphore(value: 0)
        let (source, location) = (source, location)
        self.token = token
        self.finished = finished
        isCloning = true
        errorMessage = nil
        Task {
            let result = await Task.detached {
                defer { finished.signal() }
                return Result { try cloneGitRepo(source: source, location: location, sync: token) }
            }.value
            guard self.token === token else { return }
            self.token = nil
            self.finished = nil
            isCloning = false
            switch result {
                case let .success(path):
                    onCloned(path)
                case .failure(JayError.Canceled):
                    break
                case let .failure(error):
                    errorMessage = error.friendlyDescription
            }
        }
    }

    func cancel() {
        token?.cancel()
    }

    /// jj runs in its own process group, so quitting cancels the clone and waits briefly for its cleanup.
    func cancelForTermination() {
        guard let token, let finished else { return }
        token.cancel()
        _ = finished.wait(timeout: .now() + 2)
    }

    private func updateLocation() {
        syncLocation { location = cloneLocation(folder: folder, name: name) }
    }

    /// Name and location rewrite each other, so the write that started a sync is not echoed back.
    private func syncLocation(_ update: () -> Void) {
        syncingLocation = true
        update()
        syncingLocation = false
    }
}
