public enum ReviewAction: Encodable, Sendable, Equatable {
    case setFiles([Int], reviewed: Bool)
    case addNote(file: Int, side: NoteSide, line: Int, body: String, ignoreWhitespace: Bool)
    case resolveNote(id: String)

    private enum CodingKeys: String, CodingKey { case kind, files, reviewed, file, ignoreWhitespace, side, line, body, id }

    public func encode(to encoder: any Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
            case let .setFiles(files, reviewed):
                try c.encode("setFiles", forKey: .kind)
                try c.encode(files, forKey: .files)
                try c.encode(reviewed, forKey: .reviewed)
            case let .addNote(file, side, line, body, ignoreWhitespace):
                try c.encode("addNote", forKey: .kind)
                try c.encode(file, forKey: .file)
                try c.encode(side, forKey: .side)
                try c.encode(line, forKey: .line)
                try c.encode(body, forKey: .body)
                try c.encode(ignoreWhitespace, forKey: .ignoreWhitespace)
            case let .resolveNote(id):
                try c.encode("resolveNote", forKey: .kind)
                try c.encode(id, forKey: .id)
        }
    }
}
