import AppKit
import JayJayCore
import SwiftUI

struct TagMenuItems: View {
    let name: String
    let isOnRemote: Bool
    let actions: (any TagActions)?
    let onRequest: ((DAGRequest) -> Void)?

    var body: some View {
        if !isOnRemote {
            Button(Localized.Menu.push) { actions?.gitPushTag(name: name) }
            Divider()
        }
        Button(Localized.Menu.copyTagName) {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(name, forType: .string)
        }
        Divider()
        Button(Localized.Menu.deleteTag, role: .destructive) { actions?.deleteTag(name: name) }
        if isOnRemote {
            Button(Localized.Menu.deleteTagOnRemote, role: .destructive) { onRequest?(.deleteTagOnRemote(name: name)) }
        }
    }
}
