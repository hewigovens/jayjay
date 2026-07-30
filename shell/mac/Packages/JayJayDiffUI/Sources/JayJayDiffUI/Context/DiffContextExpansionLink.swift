import AppKit
import JayJayCore

enum DiffContextExpansionLink {
    static let showMoreCount: UInt32 = 10
    static let showMoreLabel = "Show\u{00A0}\(showMoreCount)"
    static let showAllLabel = "Show\u{00A0}all"
    private static let showMoreActionValue = "show-more"
    private static let showAllActionValue = "show-all"

    static func url(for request: DiffContextExpansionRequest) -> URL {
        var components = URLComponents()
        components.scheme = DeepLink.scheme
        components.host = DeepLink.Host.diffContext
        components.path = "/expand/\(request.regionId)"
        components.queryItems = switch request.action {
            case let .showMore(lineCount):
                [
                    URLQueryItem(name: "action", value: Self.showMoreActionValue),
                    URLQueryItem(name: "count", value: String(lineCount))
                ]
            case .showAll:
                [URLQueryItem(name: "action", value: Self.showAllActionValue)]
            case .showAllRegions:
                [URLQueryItem(name: "action", value: "show-all-regions")]
        }
        // All fields are fixed ASCII or decimal integers, so URL construction cannot fail.
        return components.url!
    }

    static func request(from link: Any) -> DiffContextExpansionRequest? {
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
                return DiffContextExpansionRequest(
                    regionId: regionId,
                    action: .showMore(lineCount: count)
                )
            case Self.showAllActionValue:
                return DiffContextExpansionRequest(regionId: regionId, action: .showAll)
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
                request: DiffContextExpansionRequest(
                    regionId: region.id,
                    action: .showMore(lineCount: showMoreCount)
                ),
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
            request: DiffContextExpansionRequest(regionId: region.id, action: .showAll),
            font: font,
            color: foregroundColor,
            to: result
        )
        result.append(NSAttributedString(string: "\n", attributes: [.font: font]))
        return result
    }

    private static func appendLink(
        _ title: String,
        request: DiffContextExpansionRequest,
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
                .link: url(for: request)
            ]
        ))
    }
}
