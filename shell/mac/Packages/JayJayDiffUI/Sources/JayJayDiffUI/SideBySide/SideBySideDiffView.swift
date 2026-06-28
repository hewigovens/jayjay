import AppKit
import JayJayCore
import SwiftUI

/// GitHub Desktop-style two-column diff: left = old, right = new, synced scroll.
public struct SideBySideDiffView: View {
    public let diff: FileDiff

    public init(diff: FileDiff) {
        self.diff = diff
    }

    public var body: some View {
        SideBySideRepresentable(diff: diff)
    }
}
