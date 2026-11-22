import JayJayCore

extension SyntaxToken {
    /// From `jj_diff::SyntaxToken`'s name as the review sends it; nil for one this version doesn't know.
    init?(wire: String) {
        switch wire.lowercased() {
            case "plain": self = .plain
            case "keyword": self = .keyword
            case "stringlit": self = .stringLit
            case "comment": self = .comment
            case "number": self = .number
            case "type": self = .type
            case "function": self = .function
            case "variable": self = .variable
            case "operator": self = .operator
            case "punctuation": self = .punctuation
            case "attribute": self = .attribute
            default: return nil
        }
    }
}
