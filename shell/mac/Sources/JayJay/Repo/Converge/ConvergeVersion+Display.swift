import Foundation
import JayJayCore

extension ConvergeVersion {
    var subject: String {
        description.split(separator: "\n", maxSplits: 1).first.map(String.init) ?? ""
    }

    func value(of item: ConvergeItem) -> String? {
        switch item {
            case .description: description
            case .author: "\(author.identity) · \(author.date.formatted(date: .abbreviated, time: .standard))"
            case .parents: Self.display(parents: parents)
            case .contents: nil
        }
    }

    static func display(parents: [String]) -> String {
        parents.map { String($0.prefix(12)) }.joined(separator: ", ")
    }
}

extension ConvergeAuthor {
    var identity: String {
        "\(name) <\(email)>"
    }

    var date: Date {
        Date(timeIntervalSince1970: TimeInterval(timestampMillis) / 1000)
    }
}
