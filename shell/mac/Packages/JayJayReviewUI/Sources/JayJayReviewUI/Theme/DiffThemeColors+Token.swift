import JayJayCore

extension DiffThemeColors {
    func color(of token: RenderedDiff.Token?) -> UInt32? {
        switch token {
            case .keyword: tokKeyword
            case .string: tokString
            case .comment: tokComment
            case .number: tokNumber
            case .type: tokType
            case .other, nil: nil
        }
    }
}
