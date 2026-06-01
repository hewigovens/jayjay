import Foundation

struct ConfigSection: Identifiable {
    let name: String
    let entries: [ConfigEntry]
    var id: String {
        name
    }

    static func parse(_ raw: String) -> [ConfigSection] {
        var grouped: [(name: String, entries: [ConfigEntry])] = []
        var currentSection = ""
        var currentEntries: [ConfigEntry] = []

        for line in raw.split(separator: "\n") {
            let parts = line.split(separator: "=", maxSplits: 1)
            guard parts.count == 2 else { continue }
            let fullKey = parts[0].trimmingCharacters(in: .whitespaces)
            let value = parts[1].trimmingCharacters(in: .whitespaces)

            let section: String
            let key: String
            if let dotIndex = fullKey.firstIndex(of: ".") {
                section = String(fullKey[..<dotIndex])
                key = String(fullKey[fullKey.index(after: dotIndex)...])
            } else {
                section = "general"
                key = fullKey
            }

            if section != currentSection {
                if !currentEntries.isEmpty {
                    grouped.append((name: currentSection, entries: currentEntries))
                }
                currentSection = section
                currentEntries = []
            }
            currentEntries.append(ConfigEntry(key: key, value: value))
        }
        if !currentEntries.isEmpty {
            grouped.append((name: currentSection, entries: currentEntries))
        }

        return grouped.map { ConfigSection(name: $0.name, entries: $0.entries) }
    }
}

struct ConfigEntry: Identifiable {
    let key: String
    let value: String
    var id: String {
        key
    }

    var icon: String {
        switch key {
            case "name": "person"
            case "email": "envelope"
            case "hostname": "desktopcomputer"
            case "username": "person.badge.key"
            case "backend": "lock.shield"
            case "behavior": "signature"
            case "key": "key"
            case _ where key.contains("command"): "terminal"
            case _ where key.contains("pattern"): "doc.text.magnifyingglass"
            case _ where key.contains("sign"): "signature"
            default: "gearshape"
        }
    }
}
