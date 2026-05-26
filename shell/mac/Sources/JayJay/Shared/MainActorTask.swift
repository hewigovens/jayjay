import Foundation

enum MainActorTask {
    static func detached<Output>(
        _ operation: @escaping () throws -> Output,
        completion: @escaping @MainActor (Result<Output, any Error>) -> Void
    ) {
        Task.detached {
            let result = Result { try operation() }
            await MainActor.run {
                completion(result)
            }
        }
    }
}
