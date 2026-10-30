import Foundation
import JayJayCore

extension String {
    var isSSHLocation: Bool {
        sshMachine(location: self) != nil
    }

    var repositoryLocationURL: URL {
        if isSSHLocation, let url = URL(string: self) {
            return url
        }
        return URL(fileURLWithPath: self)
    }

    var standardizedRepositoryLocation: String {
        repositoryLocationURL.standardizedRepositoryLocationPath
    }

    var repositoryLocationTitle: String {
        let name = repositoryLocationURL.repositoryDisplayName
        return sshMachine(location: self).map { "\(name) · \($0)" } ?? name
    }

    var repositoryLocationLabel: String {
        guard let machine = sshMachine(location: self) else { return self }
        return "\(machine) · \(sshPath(location: self))"
    }
}

extension URL {
    var repositoryLocationPath: String {
        scheme == "ssh" ? absoluteString : path
    }

    var standardizedRepositoryLocationPath: String {
        scheme == "ssh" ? absoluteString : standardizedFileURL.path
    }
}
