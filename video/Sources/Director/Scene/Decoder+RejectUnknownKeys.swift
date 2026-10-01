extension Decoder {
    func rejectUnknownKeys(allowed: Set<String>) throws {
        let unknown = try container(keyedBy: AnyKey.self).allKeys.map(\.stringValue).filter { !allowed.contains($0) }
        guard unknown.isEmpty else {
            throw DecodingError.dataCorrupted(.init(
                codingPath: codingPath,
                debugDescription: "unknown key \(unknown.sorted().joined(separator: ", ")); expected \(allowed.sorted().joined(separator: ", "))"
            ))
        }
    }
}
