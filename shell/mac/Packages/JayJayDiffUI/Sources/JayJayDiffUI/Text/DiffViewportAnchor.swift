import AppKit
import JayJayCore

struct DiffViewportLineIdentity: Hashable {
    let oldLine: UInt32?
    let newLine: UInt32?
    let contextRegionId: UInt32?
    let fallbackOldLine: UInt32?
    let fallbackNewLine: UInt32?
    let continuation: UInt32

    static func unified(_ line: DiffLine) -> DiffViewportLineIdentity? {
        if let region = line.contextRegion {
            return contextRegion(region)
        }
        // Conflict summaries all project as (nil, nil); anchoring one would restore to the first match.
        if line.oldLineNo == nil, line.newLineNo == nil {
            return nil
        }
        return regular(oldLine: line.oldLineNo, newLine: line.newLineNo)
    }

    static func regular(
        oldLine: UInt32?,
        newLine: UInt32?,
        continuation: UInt32 = 0
    ) -> DiffViewportLineIdentity {
        DiffViewportLineIdentity(
            oldLine: oldLine,
            newLine: newLine,
            contextRegionId: nil,
            fallbackOldLine: nil,
            fallbackNewLine: nil,
            continuation: continuation
        )
    }

    static func contextRegion(
        _ region: ContextRegion,
        continuation: UInt32 = 0
    ) -> DiffViewportLineIdentity {
        DiffViewportLineIdentity(
            oldLine: nil,
            newLine: nil,
            contextRegionId: region.id,
            fallbackOldLine: region.oldStartLine,
            fallbackNewLine: region.newStartLine,
            continuation: continuation
        )
    }

    var revealNewLine: UInt32? {
        newLine
    }

    var fallback: DiffViewportLineIdentity? {
        guard contextRegionId != nil else { return nil }
        return .regular(
            oldLine: fallbackOldLine,
            newLine: fallbackNewLine,
            continuation: 0
        )
    }
}

struct DiffViewportLineLocation {
    let identity: DiffViewportLineIdentity
    let characterRange: NSRange
}

struct DiffViewportAnchor {
    let identity: DiffViewportLineIdentity
    let offsetFromVisibleTop: CGFloat
}
