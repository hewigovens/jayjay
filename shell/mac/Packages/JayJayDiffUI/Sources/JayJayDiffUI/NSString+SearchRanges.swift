import Foundation

extension NSString {
    func firstRange(of query: String, from start: Int) -> NSRange? {
        firstRange(of: query, from: start, to: length)
    }

    func firstRange(of query: String, from start: Int, to end: Int) -> NSRange? {
        let safeStart = max(0, min(start, length))
        let safeEnd = max(safeStart, min(end, length))
        let range = NSRange(location: safeStart, length: safeEnd - safeStart)
        let found = self.range(of: query, options: [.caseInsensitive], range: range)
        return found.location == NSNotFound ? nil : found
    }

    func lastRange(of query: String, upTo end: Int) -> NSRange? {
        let safeEnd = max(0, min(end, length))
        guard safeEnd > 0 else { return nil }
        let found = range(
            of: query,
            options: [.caseInsensitive, .backwards],
            range: NSRange(location: 0, length: safeEnd)
        )
        return found.location == NSNotFound ? nil : found
    }

    func lastRange(of query: String, from start: Int) -> NSRange? {
        let safeStart = max(0, min(start, length))
        guard safeStart < length else { return nil }
        let found = range(
            of: query,
            options: [.caseInsensitive, .backwards],
            range: NSRange(location: safeStart, length: length - safeStart)
        )
        return found.location == NSNotFound ? nil : found
    }
}
