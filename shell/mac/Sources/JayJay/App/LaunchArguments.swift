import Foundation

enum LaunchArguments {
    static func repoPath(from arguments: [String]) -> String? {
        var iterator = arguments.dropFirst().makeIterator()

        while let argument = iterator.next() {
            switch argument {
                case "--repo", "-r":
                    guard let path = iterator.next() else {
                        return nil
                    }
                    return normalizedRepoPath(path)
                case let value where value.hasPrefix("--repo="):
                    return normalizedRepoPath(String(value.dropFirst("--repo=".count)))
                case "--":
                    guard let path = iterator.next() else {
                        return nil
                    }
                    return normalizedRepoPath(path)
                case let value where value.hasPrefix("-"):
                    _ = iterator.next()
                    continue
                default:
                    return normalizedRepoPath(argument)
            }
        }

        return nil
    }

    private static func normalizedRepoPath(_ path: String) -> String {
        let cwd = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
        let url = URL(fileURLWithPath: path, relativeTo: cwd)
        return url.standardizedFileURL.path
    }
}
