import Foundation

enum HelpFeatureIndex {
    static let bundled: [HelpFeature] = load()

    static func load(bundle: Bundle = Bundle(for: HelpFeatureBundleToken.self)) -> [HelpFeature] {
        guard let url = bundle.url(forResource: "HelpFeatures", withExtension: "json"),
              let data = try? Data(contentsOf: url)
        else {
            return []
        }
        return decode(data: data)
    }

    static func decode(data: Data) -> [HelpFeature] {
        (try? JSONDecoder().decode([HelpFeature].self, from: data)) ?? []
    }
}

private final class HelpFeatureBundleToken {}
