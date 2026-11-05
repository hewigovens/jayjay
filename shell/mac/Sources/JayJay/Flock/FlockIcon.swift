import AppKit
import JayJayCore

extension FlockIcon: Identifiable {
    @MainActor private static var images: [String: NSImage] = [:]

    static var allCases: [FlockIcon] {
        flockIcons()
    }

    public var id: String {
        flockIconId(icon: self)
    }

    var title: String {
        flockIconTitle(icon: self)
    }

    @MainActor
    func image(dark: Bool) -> NSImage {
        let prefix = self == .classic ? "jayjay-dock" : "flock-\(id)"
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
