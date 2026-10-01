import CoreGraphics
import Foundation

struct Scene {
    var name = ""
    var setup: [String]?
    var repo: String?
    var rows: String?
    var window: CGRect?
    var env: [String: String] = [:]
    var defaults: [String: LaunchValue] = [:]
    var prelude: [Step] = []
    var steps: [Step] = []

    static func load(_ url: URL) throws -> Scene {
        try load(url, visited: [])
    }

    static func load(_ urls: [URL]) throws -> [Scene] {
        let scenes = try urls.map(load)
        if let reserved = scenes.first(where: { scene in [".partial", ".failed"].contains { scene.name.hasSuffix($0) } }) {
            throw Failure("\(reserved.name).json ends like a take's partial or failed output; rename it")
        }
        let shared = Dictionary(grouping: scenes) { $0.name.decomposedStringWithCanonicalMapping.lowercased() }.filter { $0.value.count > 1 }.keys.sorted()
        guard shared.isEmpty else { throw Failure("several scenes are named \(shared.joined(separator: ", ")), so their clips would overwrite each other; rename the files") }
        return scenes
    }

    private static func load(_ url: URL, visited: Set<URL>) throws -> Scene {
        let url = url.standardizedFileURL
        guard !visited.contains(url) else { throw Failure("\(url.lastPathComponent) extends itself through \(visited.map(\.lastPathComponent).sorted().joined(separator: ", "))") }
        let file: File
        do {
            file = try JSONDecoder().decode(File.self, from: Data(contentsOf: url))
        } catch let error as DecodingError {
            throw Failure("\(url.lastPathComponent): \(error.summary)")
        }
        var scene = try file.extends.map { try load(url.deletingLastPathComponent().appendingPathComponent($0), visited: visited.union([url])) } ?? Scene()
        scene.name = url.deletingPathExtension().lastPathComponent
        scene.setup = file.setup ?? scene.setup
        scene.repo = file.repo ?? scene.repo
        scene.rows = file.rows ?? scene.rows
        if let window = file.window {
            guard window.count == 4 else { throw Failure("\(url.lastPathComponent): window takes [x, y, width, height]") }
            scene.window = CGRect(x: window[0], y: window[1], width: window[2], height: window[3])
        }
        scene.env.merge(file.env ?? [:]) { $1 }
        scene.defaults.merge(file.defaults ?? [:]) { $1 }
        scene.prelude = file.prelude ?? scene.prelude
        scene.steps = file.steps ?? scene.steps
        return scene
    }

    private struct File: Decodable {
        let extends: String?
        let setup: [String]?
        let repo: String?
        let rows: String?
        let window: [Double]?
        let env: [String: String]?
        let defaults: [String: LaunchValue]?
        let prelude: [Step]?
        let steps: [Step]?

        private enum CodingKeys: String, CodingKey, CaseIterable {
            case extends, setup, repo, rows, window, env, defaults, prelude, steps
        }

        init(from decoder: Decoder) throws {
            try decoder.rejectUnknownKeys(allowed: Set(CodingKeys.allCases.map(\.rawValue)))
            let container = try decoder.container(keyedBy: CodingKeys.self)
            extends = try container.decodeIfPresent(String.self, forKey: .extends)
            setup = try container.decodeIfPresent([String].self, forKey: .setup)
            repo = try container.decodeIfPresent(String.self, forKey: .repo)
            rows = try container.decodeIfPresent(String.self, forKey: .rows)
            window = try container.decodeIfPresent([Double].self, forKey: .window)
            env = try container.decodeIfPresent([String: String].self, forKey: .env)
            defaults = try container.decodeIfPresent([String: LaunchValue].self, forKey: .defaults)
            prelude = try container.decodeIfPresent([Step].self, forKey: .prelude)
            steps = try container.decodeIfPresent([Step].self, forKey: .steps)
        }
    }
}
