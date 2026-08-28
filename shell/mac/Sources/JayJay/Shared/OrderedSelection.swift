import AppKit

enum OrderedSelectionClick {
    case replace
    case toggle
    case extend

    init(modifiers: NSEvent.ModifierFlags) {
        let modifiers = modifiers.intersection(.deviceIndependentFlagsMask)
        if modifiers.contains(.command) {
            self = .toggle
        } else if modifiers.contains(.shift) {
            self = .extend
        } else {
            self = .replace
        }
    }
}

struct OrderedSelection<ID: Hashable> {
    private(set) var selectedIDs: Set<ID>
    private(set) var primaryID: ID?
    private(set) var anchorID: ID?

    init(
        selectedIDs: Set<ID> = [],
        primaryID: ID? = nil,
        anchorID: ID? = nil
    ) {
        var selectedIDs = selectedIDs
        if let primaryID {
            selectedIDs.insert(primaryID)
        }
        self.selectedIDs = selectedIDs
        self.primaryID = primaryID
        self.anchorID = anchorID ?? primaryID
    }

    var count: Int {
        selectedIDs.count
    }

    func contains(_ id: ID) -> Bool {
        selectedIDs.contains(id)
    }

    func orderedIDs(in order: [ID]) -> [ID] {
        order.filter(selectedIDs.contains)
    }

    func formsContiguousRange(in order: [ID]) -> Bool {
        let indices = order.indices.filter { selectedIDs.contains(order[$0]) }
        guard indices.count == selectedIDs.count,
              let first = indices.first,
              let last = indices.last
        else { return false }
        return last - first + 1 == indices.count
    }

    mutating func apply(_ click: OrderedSelectionClick, to id: ID, orderedIDs: [ID]) {
        switch click {
            case .replace:
                selectOnly(id)
            case .toggle:
                toggle(id, orderedIDs: orderedIDs)
            case .extend:
                extend(to: id, orderedIDs: orderedIDs)
        }
    }

    private mutating func selectOnly(_ id: ID) {
        selectedIDs = [id]
        primaryID = id
        anchorID = id
    }

    private mutating func toggle(_ id: ID, orderedIDs: [ID]) {
        guard selectedIDs.remove(id) != nil else {
            selectedIDs.insert(id)
            primaryID = id
            anchorID = id
            return
        }
        guard !selectedIDs.isEmpty else {
            primaryID = nil
            anchorID = nil
            return
        }
        if primaryID == id || primaryID == nil {
            primaryID = self.orderedIDs(in: orderedIDs).first
        }
        if anchorID == id {
            anchorID = primaryID
        }
    }

    private mutating func extend(to id: ID, orderedIDs: [ID]) {
        let anchor = anchorID ?? primaryID ?? id
        guard let anchorIndex = orderedIDs.firstIndex(of: anchor),
              let idIndex = orderedIDs.firstIndex(of: id)
        else {
            selectOnly(id)
            return
        }
        let bounds = min(anchorIndex, idIndex) ... max(anchorIndex, idIndex)
        selectedIDs = Set(orderedIDs[bounds])
        primaryID = id
        anchorID = anchor
    }
}
