/// The session binds the backend's immutable source revision; a file index belongs to that session.
public struct ReviewSelection: Sendable, Equatable {
    public var session: Int
    public var file: Int
    public var options: DiffOptions

    public init(session: Int, file: Int, options: DiffOptions) {
        self.session = session
        self.file = file
        self.options = options
    }
}
