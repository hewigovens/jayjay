import Foundation

final class CueLog {
    private let clip: CGRect
    private let start: Date
    private var lines: [String] = []

    init(clip: CGRect, start: Date) {
        self.clip = clip
        self.start = start
    }

    func add(_ label: String, at point: CGPoint?) {
        var entry: [String: Any] = ["t": Date().timeIntervalSince(start), "label": label]
        if let point {
            entry["x"] = (point.x - clip.minX) / clip.width
            entry["y"] = (point.y - clip.minY) / clip.height
        }
        if let data = try? JSONSerialization.data(withJSONObject: entry, options: [.sortedKeys]) {
            lines.append(String(decoding: data, as: UTF8.self))
        }
    }

    func write(to url: URL) throws {
        try lines.map { $0 + "\n" }.joined().write(to: url, atomically: true, encoding: .utf8)
    }
}
