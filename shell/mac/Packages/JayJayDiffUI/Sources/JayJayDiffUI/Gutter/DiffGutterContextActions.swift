public protocol DiffGutterContextActions {
    var currentSelectedLineRange: ClosedRange<Int>? { get }

    func didSelectLines(_ lineRange: ClosedRange<Int>)
}
