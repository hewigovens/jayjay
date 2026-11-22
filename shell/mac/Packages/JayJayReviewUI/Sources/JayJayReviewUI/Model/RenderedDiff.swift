public struct RenderedDiff: Decodable, Sendable {
    public var rows: [Row]

    public init(rows: [Row]) {
        self.rows = rows
    }

    public struct Row: Decodable, Sendable {
        public var cells: [Cell]

        public init(cells: [Cell]) {
            self.cells = cells
        }
    }

    public enum Style: String, Decodable, Sendable {
        case context, added, removed, unchanged, separator, other

        public init(from decoder: any Decoder) throws {
            self = try Style(rawValue: decoder.singleValueContainer().decode(String.self)) ?? .other
        }
    }

    public enum Token: String, Decodable, Sendable {
        case keyword, string = "stringlit", comment, number, type, other

        public init(from decoder: any Decoder) throws {
            self = try Token(rawValue: decoder.singleValueContainer().decode(String.self).lowercased()) ?? .other
        }
    }

    public struct Span: Decodable, Sendable, Equatable {
        public var text: String
        /// A span styled like its cell is the changed part of the line.
        public var style: Style?
        public var token: Token?

        public init(text: String, style: Style? = nil, token: Token? = nil) {
            self.text = text
            self.style = style
            self.token = token
        }
    }

    public struct Cell: Decodable, Sendable, Equatable {
        public var oldLine: Gutter?
        public var newLine: Int?
        public var style: Style
        public var spans: [Span]
        public var noEofNewline: Bool

        public var text: String {
            spans.map(\.text).joined()
        }

        public init(oldLine: Gutter? = nil, newLine: Int? = nil, style: Style, spans: [Span], noEofNewline: Bool = false) {
            self.oldLine = oldLine
            self.newLine = newLine
            self.style = style
            self.spans = spans
            self.noEofNewline = noEofNewline
        }
    }

    public enum Gutter: Decodable, Sendable, Equatable {
        case number(Int), label(String)

        public init(from decoder: any Decoder) throws {
            let c = try decoder.singleValueContainer()
            if let n = try? c.decode(Int.self) {
                self = .number(n)
            } else {
                self = try .label(c.decode(String.self))
            }
        }

        public var text: String {
            switch self {
                case let .number(n): String(n)
                case let .label(s): s
            }
        }
    }
}
