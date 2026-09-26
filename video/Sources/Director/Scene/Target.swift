import Foundation

struct Target: Decodable {
    var id: String?
    var idPrefix: String?
    var label: String?
    var role: String?
    var menuItem: String?
    var row: String?
    var lane: String?
    var file: String?
    var line: NSRegularExpression?
    var x: Double?
    var y: Double?
    var index = 0

    init(menuItem: String) {
        self.menuItem = menuItem
    }

    private enum CodingKeys: String, CodingKey, CaseIterable {
        case id, idPrefix, label, role, menuItem, row, lane, file, line, x, y, index
    }

    init(from decoder: Decoder) throws {
        if let id = try? decoder.singleValueContainer().decode(String.self) {
            self.id = id
            return
        }
        try decoder.rejectUnknownKeys(allowed: Set(CodingKeys.allCases.map(\.rawValue)))
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decodeIfPresent(String.self, forKey: .id)
        idPrefix = try container.decodeIfPresent(String.self, forKey: .idPrefix)
        label = try container.decodeIfPresent(String.self, forKey: .label)
        role = try container.decodeIfPresent(String.self, forKey: .role).map { $0.hasPrefix("AX") ? $0 : "AX" + $0.prefix(1).uppercased() + $0.dropFirst() }
        menuItem = try container.decodeIfPresent(String.self, forKey: .menuItem)
        row = try container.decodeIfPresent(String.self, forKey: .row)
        lane = try container.decodeIfPresent(String.self, forKey: .lane)
        file = try container.decodeIfPresent(String.self, forKey: .file)
        if let pattern = try container.decodeIfPresent(String.self, forKey: .line) {
            do {
                line = try NSRegularExpression(pattern: pattern)
            } catch {
                throw DecodingError.dataCorruptedError(forKey: .line, in: container, debugDescription: "not a regular expression: \(pattern)")
            }
        }
        x = try container.decodeIfPresent(Double.self, forKey: .x)
        y = try container.decodeIfPresent(Double.self, forKey: .y)
        index = try container.decodeIfPresent(Int.self, forKey: .index) ?? 0
        guard [id, idPrefix, label, role, menuItem, row, lane, file].contains(where: { $0 != nil }) else {
            throw DecodingError.dataCorrupted(.init(
                codingPath: decoder.codingPath,
                debugDescription: "a target needs id, idPrefix, label, role, menuItem, row, lane, or file"
            ))
        }
    }

    var name: String {
        menuItem ?? row ?? lane ?? file ?? label ?? id ?? idPrefix ?? role ?? "element"
    }

    var summary: String {
        if let menuItem {
            return "menu item \"\(menuItem)\""
        }
        if let row {
            return "row \"\(row)\""
        }
        if let lane {
            return "lane \"\(lane)\""
        }
        if let file {
            return "file \"\(file)\""
        }
        if let label {
            return "\"\(label)\""
        }
        return id ?? idPrefix.map { "\($0)…" } ?? role ?? "element"
    }
}
