import JayJayCore

extension RepoViewModel {
    func createTag(name: String, rev: String) {
        perform(selecting: nil) { try $0.createTag(name: name, rev: rev) }
    }

    func deleteTag(name: String) {
        perform(selecting: nil) { try $0.deleteTag(name: name) }
    }
}
