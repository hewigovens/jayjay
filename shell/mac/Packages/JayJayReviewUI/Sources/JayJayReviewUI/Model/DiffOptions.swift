public struct DiffOptions: Codable, Sendable, Equatable {
    public var split: Bool
    public var ignoreWhitespace: Bool
    public var showAll: Bool

    public init(split: Bool = false, ignoreWhitespace: Bool = false, showAll: Bool = false) {
        self.split = split
        self.ignoreWhitespace = ignoreWhitespace
        self.showAll = showAll
    }
}
