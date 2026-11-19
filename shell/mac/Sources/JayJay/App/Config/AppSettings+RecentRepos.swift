import Foundation

extension AppSettings {
    func recordOpenedRepo(_ path: String) {
        lastOpenedRepo = addRecentRepo(path)
    }

    @discardableResult
    func addRecentRepo(_ path: String) -> String {
        let standardizedPath = Self.standardizedRepositoryPath(path)
        recentRepos.removeAll(where: { $0 == standardizedPath })
        recentRepos.insert(standardizedPath, at: 0)
        recentRepos = Array(recentRepos.prefix(12))
        return standardizedPath
    }

    func removeRecentRepo(_ path: String) {
        recentRepos.removeAll(where: { $0 == path })
        if lastOpenedRepo == path {
            lastOpenedRepo = recentRepos.first
        }
    }

    static func storedRecentRepos(in defaults: UserDefaults) -> [String] {
        var seen = Set<String>()
        return (defaults.stringArray(forKey: StorageKeys.recentRepos) ?? [])
            .filter { !$0.isEmpty }
            .map(standardizedRepositoryPath)
            .filter { seen.insert($0).inserted }
    }

    static func storedLastOpenedRepo(in defaults: UserDefaults) -> String? {
        defaults.string(forKey: StorageKeys.lastOpenedRepo)
            .flatMap { $0.isEmpty ? nil : standardizedRepositoryPath($0) }
    }

    static func standardizedRepositoryPath(_ path: String) -> String {
        path.standardizedRepositoryLocation
    }
}
