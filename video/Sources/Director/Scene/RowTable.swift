import Foundation

/// Graph rows and Overview lanes expose no text, so scenes name them by subject.
struct RowTable {
    private let rows: [(id: String, subject: String)]

    init(contentsOf url: URL) throws {
        rows = try String(contentsOf: url, encoding: .utf8).split(separator: "\n").compactMap { line in
            let fields = line.split(separator: "\t", maxSplits: 1).map(String.init)
            return fields.count == 2 ? (fields[0], fields[1]) : nil
        }
    }

    func id(for text: String) throws -> String {
        let matches = rows.filter { $0.subject.contains(text) }
        guard matches.count == 1 else {
            throw Failure(matches.isEmpty ? "no row in rows.tsv mentions \"\(text)\"" : "\(matches.count) rows in rows.tsv mention \"\(text)\"; name one more precisely")
        }
        return matches[0].id
    }
}
