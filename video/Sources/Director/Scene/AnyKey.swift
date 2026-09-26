struct AnyKey: CodingKey {
    let stringValue: String
    var intValue: Int? {
        nil
    }

    init(_ stringValue: String) {
        self.stringValue = stringValue
    }

    init?(stringValue: String) {
        self.stringValue = stringValue
    }

    init?(intValue: Int) {
        nil
    }
}
