/// A value loaded for `key`; a load whose key no longer matches is stale and dropped.
struct KeyedValue<Value: Equatable>: Equatable {
    let key: String
    var value: Value?
}
