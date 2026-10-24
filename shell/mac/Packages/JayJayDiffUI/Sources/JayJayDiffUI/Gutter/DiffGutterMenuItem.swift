struct DiffGutterMenuItem {
    let title: String
    let enabled: Bool
    let action: (() -> Void)?
    var identifier: String?

    static let separator = DiffGutterMenuItem(title: "", enabled: false, action: nil)
}
