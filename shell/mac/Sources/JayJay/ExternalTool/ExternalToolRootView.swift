import JayJayCore
import SwiftUI

struct ExternalToolRootView: View {
    let invocation: ExternalToolInvocation
    let onLoadFailure: () -> Void

    var body: some View {
        switch invocation {
            case let .diff(left, right, editable):
                ExternalDiffToolView(
                    left: left,
                    right: right,
                    editable: editable,
                    onLoadFailure: onLoadFailure
                )
            case let .merge(left, base, right, output, path, markerLength):
                ExternalMergeToolView(
                    left: left,
                    base: base,
                    right: right,
                    output: output,
                    path: path,
                    markerLength: markerLength,
                    onLoadFailure: onLoadFailure
                )
        }
    }
}
