import SwiftUI

extension View {
    func selectionContextMenu(
        for row: String,
        request: Binding<SelectionContextMenuRequest?>,
        @ViewBuilder content: () -> some View
    ) -> some View {
        let menuContent = content()
        return contextMenu { menuContent }
            .background {
                SelectionContextMenu(
                    openRequest: request.wrappedValue?.id(for: row),
                    onOpen: { request.wrappedValue = nil },
                    content: menuContent
                )
                .allowsHitTesting(false)
                .accessibilityHidden(true)
            }
    }

    func selectionContextMenuExclusion() -> some View {
        background {
            SelectionContextMenuExclusion()
                .allowsHitTesting(false)
                .accessibilityHidden(true)
        }
    }
}
