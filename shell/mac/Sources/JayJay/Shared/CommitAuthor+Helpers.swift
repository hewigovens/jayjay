import JayJayCore

extension CommitAuthor {
    static func empty(timestampMillis: Int64 = 0) -> CommitAuthor {
        CommitAuthor(name: "", email: "", timestampMillis: timestampMillis)
    }
}
