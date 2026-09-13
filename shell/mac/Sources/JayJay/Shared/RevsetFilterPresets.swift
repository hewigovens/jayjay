import JayJayCore

enum RevsetFilterPresets {
    static let all = revsetPresets()

    static func preset(id: String) -> RevsetPreset? {
        all.first { $0.id == id }
    }
}
