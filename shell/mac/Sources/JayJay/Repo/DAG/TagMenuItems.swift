import AppKit
import SwiftUI

struct TagMenuItems: View {
    let name: String
    let isOnRemote: Bool
    let actions: (any TagActions)?
    let onRequest: ((DAGRequest) -> Void)?

    var body: some View {
        if !isOnRemote {
            Button("Push") { actions?.gitPushTag(name: name) }
            Divider()
        }
        Button("Copy Tag Name") {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(name, forType: .string)
        }
        Divider()
        Button("Delete Tag", role: .destructive) { actions?.deleteTag(name: name) }
        if isOnRemote {
            Button("Delete Tag on Remote", role: .destructive) { onRequest?(.deleteTagOnRemote(name: name)) }
        }
    }
}
