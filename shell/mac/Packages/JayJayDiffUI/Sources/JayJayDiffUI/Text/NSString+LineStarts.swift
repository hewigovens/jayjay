import AppKit

extension NSString {
    /// The diff counts LF boundaries; Foundation also treats CR and Unicode paragraph separators as line breaks.
    func lineStarts() -> [Int] {
        var starts: [Int] = []
        var location = 0
        while location < length {
            starts.append(location)
            let newline = range(of: "\n", range: NSRange(location: location, length: length - location))
            guard newline.location != NSNotFound else { break }
            location = NSMaxRange(newline)
        }
        return starts
    }
}
