import JayJayCore

enum ShortcutGuide {
    static func sections(bindings: [ShortcutBinding]) -> [ShortcutSection] {
        var groups: [ShortcutSection] = []
        for binding in bindings {
            if let index = groups.firstIndex(where: { $0.id == binding.contextTitle }) {
                groups[index].entries.append(binding)
            } else {
                groups.append(ShortcutSection(title: binding.contextTitle, entries: [binding]))
            }
        }
        return groups
    }

    static func columns(bindings: [ShortcutBinding]) -> [[ShortcutSection]] {
        let sections = sections(bindings: bindings.filter { !$0.chords.isEmpty })
        let half = sections.reduce(0) { $0 + $1.entries.count } / 2
        var left: [ShortcutSection] = []
        var right: [ShortcutSection] = []
        var count = 0
        for section in sections {
            if count < half {
                left.append(section)
                count += section.entries.count
            } else {
                right.append(section)
            }
        }
        return [left, right]
    }
}
