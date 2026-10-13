import Foundation

struct SelectionContextMenuRequest: Equatable {
    let target: String
    let id = UUID()

    func id(for row: String) -> UUID? {
        target == row ? id : nil
    }
}
