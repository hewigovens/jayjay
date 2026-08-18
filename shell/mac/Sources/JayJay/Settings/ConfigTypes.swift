import Foundation

struct ConfigSection: Identifiable {
    let name: String
    let entries: [ConfigEntry]
    var id: String {
        name
    }

    /// Groups by section name across the whole listing. `jj config list` is not sorted, so the same section commonly reappears non-contiguously (for example `ui.editor`, then other sections, then `ui.diff`). Adjacent-only grouping produces duplicate SwiftUI identities and Form cells reuse the wrong section.
    static func parse(_ raw: String) -> [ConfigSection] {
        var order: [String] = []
        var entriesByName: [String: [ConfigEntry]] = [:]

        for line in raw.split(whereSeparator: \.isNewline) {
            let parts = line.split(separator: "=", maxSplits: 1)
            guard parts.count == 2 else { continue }
            let fullKey = parts[0].trimmingCharacters(in: .whitespaces)
            let value = parts[1].trimmingCharacters(in: .whitespaces)
            guard !fullKey.isEmpty else { continue }

            let section: String
            let key: String
            if let dotIndex = fullKey.firstIndex(of: ".") {
                section = String(fullKey[..<dotIndex])
                key = String(fullKey[fullKey.index(after: dotIndex)...])
            } else {
                section = "general"
                key = fullKey
            }

            if entriesByName[section] == nil {
                order.append(section)
                entriesByName[section] = []
            }
            entriesByName[section, default: []].append(
                ConfigEntry(section: section, key: key, value: value)
            )
        }

        return order.map { name in
            ConfigSection(name: name, entries: entriesByName[name] ?? [])
        }
    }
}

struct ConfigEntry: Identifiable {
    let section: String
    let key: String
    let value: String
    var id: String {
        "\(section).\(key)"
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
