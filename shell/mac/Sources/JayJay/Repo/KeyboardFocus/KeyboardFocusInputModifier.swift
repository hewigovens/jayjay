import SwiftUI

extension View {
    @ViewBuilder
    func keyboardFocusInput(_ stop: KeyboardFocusStop, isAvailable: Bool = true) -> some View {
        if isAvailable {
            modifier(KeyboardFocusInputModifier(stop: stop))
        } else {
            self
        }
    }
}

private struct KeyboardFocusInputModifier: ViewModifier {
    let stop: KeyboardFocusStop
    @Environment(KeyboardFocus.self) private var focus: KeyboardFocus?
    @FocusState private var isFocused: Bool

    func body(content: Content) -> some View {
        content
            .focused($isFocused)
            .keyboardFocusStop(stop, action: requestFocus)
            .onAppear(perform: requestFocus)
            .onChange(of: isFocused) { _, isFocused in
                focus?.updateInputFocus(stop, isFocused: isFocused)
            }
    }

    private func requestFocus() {
        // Newly inserted inputs must be attached before SwiftUI can make them first responder.
        DispatchQueue.main.async {
            if focus?.control == stop {
                isFocused = true
            }
        }
    }
}
