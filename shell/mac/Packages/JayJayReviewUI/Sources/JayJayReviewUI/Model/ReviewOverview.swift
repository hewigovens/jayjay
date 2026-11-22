public struct ReviewOverview: Decodable, Sendable {
    public var title: String
    public var description: String
    public var author: String
    public var files: [ReviewFileSummary]
    public var notes: [ReviewNote]

    public init(title: String, description: String, author: String, files: [ReviewFileSummary], notes: [ReviewNote] = []) {
        self.title = title
        self.description = description
        self.author = author
        self.files = files
        self.notes = notes
    }

    private enum CodingKeys: String, CodingKey { case title, description, author, files, notes }

    public init(from decoder: any Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        title = try c.decode(String.self, forKey: .title)
        description = try c.decode(String.self, forKey: .description)
        author = try c.decode(String.self, forKey: .author)
        files = try c.decode([ReviewFileSummary].self, forKey: .files)
        notes = try c.decodeIfPresent([ReviewNote].self, forKey: .notes) ?? []
    }
}
