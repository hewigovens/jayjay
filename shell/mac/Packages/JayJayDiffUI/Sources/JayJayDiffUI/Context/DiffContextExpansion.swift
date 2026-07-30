import Foundation
import JayJayCore

public enum DiffContextExpansionAction: Hashable, Sendable {
    case showMore(lineCount: UInt32)
    case showAll
    case showAllRegions
}

public struct DiffContextExpansionRequest: Hashable, Sendable {
    public let regionId: UInt32
    public let action: DiffContextExpansionAction

    public init(regionId: UInt32, action: DiffContextExpansionAction) {
        self.regionId = regionId
        self.action = action
    }
}
