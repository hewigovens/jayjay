protocol TagActions: AnyObject {
    func deleteTag(name: String)
    func gitPushTag(name: String)
}
