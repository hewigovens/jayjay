import Foundation

extension NSString {
    /// UTF-16 location where the 1-based `line` starts, counting only LF like the diff engine; lines past the end land at the end.
    func location(ofLine line: Int) -> Int {
        var location = 0
        for _ in 1 ..< max(line, 1) {
            let newline = range(of: "\n", options: .literal, range: NSRange(location: location, length: length - location))
            guard newline.location != NSNotFound else { return length }
            location = NSMaxRange(newline)
        }
        return location
    }
}
