import Foundation

struct FileRevealRequest {
    let id = UUID()
    let revision: String
    let path: String
}
