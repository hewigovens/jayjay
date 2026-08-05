import AppKit
import JayJayCore

struct SideBySideViewportIdentityBuilder {
    private var oldLine: UInt32?
    private var newLine: UInt32?
    private var continuation: UInt32 = 0

    mutating func identity(
        for row: SideBySideRow,
        region: ContextRegion?
    ) -> DiffViewportLineIdentity? {
        if let region {
            oldLine = nil
            newLine = nil
            continuation = 0
            return .contextRegion(region)
        }
        if row.old.conflictKind != .none || row.new.conflictKind != .none {
            oldLine = nil
            newLine = nil
            continuation = 0
            return nil
        }

        let rowOldLine = UInt32(row.old.lineNo)
        let rowNewLine = UInt32(row.new.lineNo)
        if rowOldLine != nil || rowNewLine != nil {
            oldLine = rowOldLine
            newLine = rowNewLine
            continuation = 0
        } else if oldLine != nil || newLine != nil {
            continuation += 1
        } else {
            return nil
        }
        return .regular(
            oldLine: oldLine,
            newLine: newLine,
            continuation: continuation
        )
    }
}
