import AppKit
import JayJayCore

enum FlockIcon: String, CaseIterable, Identifiable {
    case classic
    case fishOwl = "fish-owl"
    case goldenPheasant = "golden-pheasant"
    case kingfisher
    case flycatcher

    @MainActor private static var images: [String: NSImage] = [:]

    var id: Self {
        self
    }

    var title: String {
        switch self {
            case .classic: Localized.Settings.flockIconClassic
            case .fishOwl: Localized.Settings.flockIconFishOwl
            case .goldenPheasant: Localized.Settings.flockIconGoldenPheasant
            case .kingfisher: Localized.Settings.flockIconKingfisher
            case .flycatcher: Localized.Settings.flockIconFlycatcher
        }
    }

    @MainActor
    func image(dark: Bool) -> NSImage {
        let prefix = self == .classic ? "jayjay-dock" : "flock-\(rawValue)"
        let resource = "\(prefix)-\(dark ? "dark" : "light")"
        if let image = Self.images[resource] {
            return image
        }
        guard let url = Bundle.main.url(forResource: resource, withExtension: "png"),
              let image = NSImage(contentsOf: url)
        else { return NSApplication.shared.applicationIconImage }
        Self.images[resource] = image
        return image
    }
}
