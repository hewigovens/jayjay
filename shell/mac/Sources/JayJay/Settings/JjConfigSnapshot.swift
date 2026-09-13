import Foundation
import JayJayCore

struct JjConfigSnapshot: Sendable {
    let path: String
    let sections: [ConfigSection]
    let error: String?

    var isMissing: Bool {
        error == nil && path.isEmpty && sections.isEmpty
    }

    init(path: String, sections: [ConfigSection], error: String? = nil) {
        self.path = path
        self.sections = sections
        self.error = error
    }

    static func load() -> Self {
        let snapshot = loadJjUserConfig()
        return Self(
            path: snapshot.path,
            sections: ConfigSection.parse(snapshot.listing),
            error: snapshot.error
        )
    }
}
