extension DecodingError {
    var summary: String {
        let context: Context
        switch self {
            case let .typeMismatch(_, found), let .valueNotFound(_, found), let .dataCorrupted(found):
                context = found
            case let .keyNotFound(key, found):
                return Self.path(found.codingPath + [key]) + ": missing"
            @unknown default:
                return localizedDescription
        }
        let path = Self.path(context.codingPath)
        return path.isEmpty ? context.debugDescription : "\(path): \(context.debugDescription)"
    }

    private static func path(_ keys: [CodingKey]) -> String {
        let path = keys.map { $0.intValue.map { "[\($0)]" } ?? ".\($0.stringValue)" }.joined()
        return path.hasPrefix(".") ? String(path.dropFirst()) : path
    }
}
