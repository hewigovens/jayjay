import AppKit

struct PasteboardSnapshot {
    private let items: [[NSPasteboard.PasteboardType: Data]]

    static func current() -> PasteboardSnapshot {
        let items = NSPasteboard.general.pasteboardItems ?? []
        return PasteboardSnapshot(items: items.map { item in
            Dictionary(uniqueKeysWithValues: item.types.compactMap { type in item.data(forType: type).map { (type, $0) } })
        })
    }

    func restore() {
        NSPasteboard.general.clearContents()
        let restored = items.map { types in
            let item = NSPasteboardItem()
            for (type, data) in types {
                item.setData(data, forType: type)
            }
            return item
        }
        NSPasteboard.general.writeObjects(restored)
    }
}
