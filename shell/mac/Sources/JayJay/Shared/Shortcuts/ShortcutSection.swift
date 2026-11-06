import JayJayCore

struct ShortcutSection: Identifiable {
    var id: String { title }
    let title: String
    var entries: [ShortcutBinding]
}
