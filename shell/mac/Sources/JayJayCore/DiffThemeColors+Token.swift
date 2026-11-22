public extension DiffThemeColors {
    /// Nil for a token drawn as the text around it.
    func color(of token: SyntaxToken) -> UInt32? {
        switch token {
            case .keyword, .operator: tokKeyword
            case .stringLit: tokString
            case .comment: tokComment
            case .number: tokNumber
            case .type, .function, .attribute: tokType
            case .plain, .variable, .punctuation: nil
        }
    }
}
