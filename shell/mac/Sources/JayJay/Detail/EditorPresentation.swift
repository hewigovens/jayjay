import SwiftUI

struct EditorPresentation<Session> {
    let id = UUID()
    let revision: String
    var session: Session?

    var isPreparing: Bool {
        session == nil
    }
}

extension Binding {
    func openSession<Session>() -> Binding<Session?> where Value == EditorPresentation<Session>? {
        Binding<Session?>(
            get: { wrappedValue?.session },
            set: {
                if $0 == nil {
                    wrappedValue = nil
                }
            }
        )
    }
}
