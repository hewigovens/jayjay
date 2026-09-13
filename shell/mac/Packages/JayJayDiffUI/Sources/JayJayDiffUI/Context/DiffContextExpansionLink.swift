import AppKit
import JayJayCore

enum DiffContextExpansionLink {
    static let showMoreCount: UInt32 = 10
    static let showMoreLabel = "Show\u{00A0}\(showMoreCount)"
    static let showAllLabel = "Show\u{00A0}all"
    private static let showMoreActionValue = "show-more"
    private static let showAllActionValue = "show-all"

    static func url(regionId: UInt32, expansion: ContextExpansion) -> URL {
        var components = URLComponents()
        components.scheme = DeepLink.scheme
        components.host = DeepLink.Host.diffContext
        components.path = "/expand/\(regionId)"
        components.queryItems = switch expansion {
            case let .showMore(lineCount):
                [
                    URLQueryItem(name: "action", value: Self.showMoreActionValue),
                    URLQueryItem(name: "count", value: String(lineCount))
                ]
            case .showAll:
                [URLQueryItem(name: "action", value: Self.showAllActionValue)]
        }
        // All fields are fixed ASCII or decimal integers, so URL construction cannot fail.
        return components.url!
    }

    static func request(from link: Any) -> ContextExpansionRequest? {
        let url: URL? = switch link {
            case let value as URL:
                value
            case let value as NSURL:
                value as URL
            case let value as String:
                URL(string: value)
            default:
                nil
        }
        guard let url,
              let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
              components.scheme == DeepLink.scheme,
              components.host == DeepLink.Host.diffContext,
              components.path.hasPrefix("/expand/"),
              let regionId = UInt32(components.path.dropFirst("/expand/".count))
        else { return nil }

        let query = Dictionary(
            uniqueKeysWithValues: (components.queryItems ?? []).compactMap { item in
                item.value.map { (item.name, $0) }
            }
        )
        switch query["action"] {
            case Self.showMoreActionValue:
                guard let count = query["count"].flatMap(UInt32.init), count > 0 else { return nil }
                return .region(regionId: regionId, expansion: .showMore(lineCount: count))
            case Self.showAllActionValue:
                return .region(regionId: regionId, expansion: .showAll)
            default:
                return nil
        }
    }

    static func separatorString(
        text: String,
        region: ContextRegion?,
        enablesExpansion: Bool,
        font: NSFont,
        foregroundColor: NSColor
    ) -> NSAttributedString {
        if let region, enablesExpansion {
            return attributedSeparator(
                text: text, region: region, font: font, foregroundColor: foregroundColor
            )
        }
        return NSAttributedString(
            string: "⋯ \(text)\n",
            attributes: [.font: font, .foregroundColor: foregroundColor]
        )
    }

    static func attributedSeparator(
        text: String,
        region: ContextRegion,
        font: NSFont,
        foregroundColor: NSColor
    ) -> NSAttributedString {
        let result = NSMutableAttributedString(
            string: "⋯ \(text)  ",
            attributes: [
                .font: font,
                .foregroundColor: foregroundColor
            ]
        )
        if region.initialLineCount > showMoreCount {
            appendLink(
                "Show\u{00A0}10",
                url: url(regionId: region.id, expansion: .showMore(lineCount: showMoreCount)),
                font: font,
                color: foregroundColor,
                to: result
            )
            result.append(NSAttributedString(
                string: "  ",
                attributes: [.font: font, .foregroundColor: foregroundColor]
            ))
        }
        appendLink(
            showAllLabel,
            url: url(regionId: region.id, expansion: .showAll),
            font: font,
            color: foregroundColor,
            to: result
        )
        result.append(NSAttributedString(string: "\n", attributes: [.font: font]))
        return result
    }

    private static func appendLink(
        _ title: String,
        url: URL,
        font: NSFont,
        color: NSColor,
        to result: NSMutableAttributedString
    ) {
        result.append(NSAttributedString(
            string: title,
            attributes: [
                .font: font,
                .foregroundColor: color,
                .cursor: NSCursor.pointingHand,
                .link: url
            ]
        ))
    }
}
