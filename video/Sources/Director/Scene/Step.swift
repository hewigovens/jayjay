import Foundation

struct Step: Decodable {
    enum Action {
        case click(Target)
        case doubleClick(Target)
        case rightClick(Target)
        case key(KeyCombo)
        case paste(String)
        case wait(Target)
        case waitGone(Target)
        case beat(TimeInterval)
    }

    let action: Action
    let label: String?
    let timeout: TimeInterval

    private static let cued: Set = ["click", "doubleClick", "rightClick", "menu", "key", "paste"]
    private static let waiting: Set = ["click", "doubleClick", "rightClick", "menu", "wait", "waitGone"]
    private static let actions = ["click", "doubleClick", "rightClick", "menu", "key", "paste", "wait", "waitGone", "beat"]

    init(from decoder: Decoder) throws {
        try decoder.rejectUnknownKeys(allowed: Set(Self.actions + ["label", "timeout"]))
        let container = try decoder.container(keyedBy: AnyKey.self)
        let named = container.allKeys.filter { Self.actions.contains($0.stringValue) }
        guard named.count == 1, let key = named.first else {
            throw DecodingError.dataCorrupted(.init(
                codingPath: decoder.codingPath,
                debugDescription: "a step takes exactly one of \(Self.actions.joined(separator: ", "))"
            ))
        }
        for (option, allowed) in [("label", Self.cued), ("timeout", Self.waiting)] where container.contains(AnyKey(option)) && !allowed.contains(key.stringValue) {
            throw DecodingError.dataCorruptedError(forKey: AnyKey(option), in: container, debugDescription: "\(key.stringValue) takes no \(option)")
        }
        label = try container.decodeIfPresent(String.self, forKey: AnyKey("label"))
        timeout = try container.decodeIfPresent(TimeInterval.self, forKey: AnyKey("timeout")) ?? 10
        switch key.stringValue {
            case "click":
                action = try .click(container.decode(Target.self, forKey: key))
            case "doubleClick":
                action = try .doubleClick(container.decode(Target.self, forKey: key))
            case "rightClick":
                action = try .rightClick(container.decode(Target.self, forKey: key))
            case "menu":
                action = try .click(Target(menuItem: container.decode(String.self, forKey: key)))
            case "key":
                let text = try container.decode(String.self, forKey: key)
                do {
                    action = try .key(KeyCombo(text))
                } catch {
                    throw DecodingError.dataCorruptedError(forKey: key, in: container, debugDescription: "\(error)")
                }
            case "paste":
                action = try .paste(container.decode(String.self, forKey: key))
            case "wait":
                action = try .wait(container.decode(Target.self, forKey: key))
            case "waitGone":
                action = try .waitGone(container.decode(Target.self, forKey: key))
            default:
                action = try .beat(container.decode(TimeInterval.self, forKey: key))
        }
    }

    var summary: String {
        switch action {
            case let .click(target): "click \(target.summary)"
            case let .doubleClick(target): "double-click \(target.summary)"
            case let .rightClick(target): "right-click \(target.summary)"
            case let .key(combo): "key \(combo.text)"
            case .paste: "paste"
            case let .wait(target): "wait for \(target.summary)"
            case let .waitGone(target): "wait for \(target.summary) to go"
            case let .beat(seconds): "beat \(seconds)"
        }
    }
}
