import SwiftUI

struct PaletteKeyNavigation: ViewModifier {
    let onMove: (Int) -> Void
    let onEscape: () -> Void

    func body(content: Content) -> some View {
        content
            .onKeyPress(.upArrow) {
                onMove(-1)
                return .handled
            }
            .onKeyPress(.downArrow) {
                onMove(1)
                return .handled
            }
            .onKeyPress { press in
                guard press.modifiers.contains(.control) else { return .ignored }
                switch press.characters {
                    case "p": onMove(-1)
                    case "n": onMove(1)
                    default: return .ignored
                }
                return .handled
            }
            .onKeyPress(.escape) {
                onEscape()
                return .handled
            }
    }
}

extension View {
    func paletteKeyNavigation(onMove: @escaping (Int) -> Void, onEscape: @escaping () -> Void) -> some View {
        modifier(PaletteKeyNavigation(onMove: onMove, onEscape: onEscape))
    }
}
