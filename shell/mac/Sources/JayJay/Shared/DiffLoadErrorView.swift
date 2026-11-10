import JayJayCore
import SwiftUI

struct DiffLoadErrorView: View {
    let message: String
    let retry: () -> Void

    var body: some View {
        ContentUnavailableView {
            Label(Localized.Repo.couldntLoadDiff, systemImage: "exclamationmark.triangle")
        } description: {
            Text(message)
        } actions: {
            Button(Localized.Repo.retry, action: retry)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
