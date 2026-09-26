import JayJayCore

extension JjConfigEntry {
    var icon: String {
        switch key {
            case "name": "person"
            case "email": "envelope"
            case "hostname": "desktopcomputer"
            case "username": "person.badge.key"
            case "backend": "lock.shield"
            case "behavior": "signature"
            case "key": "key"
            case _ where key.contains("command"): "terminal"
            case _ where key.contains("pattern"): "doc.text.magnifyingglass"
            case _ where key.contains("sign"): "signature"
            default: "gearshape"
        }
    }
}
