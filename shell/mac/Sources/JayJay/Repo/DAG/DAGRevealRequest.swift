import Foundation

struct DAGRevealRequest: Equatable, Identifiable {
    let id = UUID()
    let changeId: String
}
