import JayJayCore

enum RevsetFilterPresets {
    static var all: [RevsetPreset] {
        revsetPresets()
    }

    static func preset(id: String) -> RevsetPreset? {
        all.first { $0.id == id }
    }
}
