import Foundation

extension URL {
    var repositoryDisplayName: String {
        lastPathComponent.isEmpty ? path : lastPathComponent
    }
}
