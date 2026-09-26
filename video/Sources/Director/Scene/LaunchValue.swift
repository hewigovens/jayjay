import CoreGraphics

enum LaunchValue: Decodable, Equatable {
    case bool(Bool)
    case number(Double)
    case string(String)

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let value = try? container.decode(Bool.self) {
            self = .bool(value)
        } else if let value = try? container.decode(Double.self) {
            self = .number(value)
        } else {
            self = try .string(container.decode(String.self))
        }
    }

    /// AppKit rects start bottom-left on the primary display.
    func replacingWindow(with window: CGRect?) -> LaunchValue {
        guard case let .string(value) = self, let window, value.contains("${window}") else { return self }
        let flippedY = CGDisplayBounds(CGMainDisplayID()).height - window.maxY
        return .string(value.replacingOccurrences(of: "${window}", with: "{{\(Int(window.minX)), \(Int(flippedY))}, {\(Int(window.width)), \(Int(window.height))}}"))
    }

    /// The argument domain drops an unquoted rect like `{{0, 0}, {10, 10}}`.
    var argument: String {
        switch self {
            case let .bool(value):
                value ? "YES" : "NO"
            case let .number(value):
                value == value.rounded() && abs(value) < 1e15 ? String(Int(value)) : String(value)
            case let .string(value) where !value.isEmpty && value.allSatisfy(Self.plain.contains):
                value
            case let .string(value):
                "\"" + value.replacingOccurrences(of: "\\", with: "\\\\").replacingOccurrences(of: "\"", with: "\\\"") + "\""
        }
    }

    private static let plain = Set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$+/:.-")
}
