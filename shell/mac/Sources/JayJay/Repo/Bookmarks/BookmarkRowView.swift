import JayJayCore
import SwiftUI

struct BookmarkRowView: View {
    let bookmark: BookmarkInfo
    let caption: String?

    static func caption(for bookmark: BookmarkInfo) -> String? {
        if !bookmark.trackedRemotes.isEmpty {
            return bookmark.trackedRemotes.map { "@\($0)" }.joined(separator: ", ")
        }
        if !bookmark.availableRemotes.isEmpty {
            return "Remote available: \(bookmark.availableRemotes.map { "@\($0)" }.joined(separator: ", "))"
        }
        return nil
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 1) {
            HStack(spacing: 6) {
                Text(bookmark.name)
                    .font(.system(size: 13))
                    .lineLimit(1)
                Image(systemName: bookmark.isTrackingRemote ? "cloud.fill" : "cloud.slash")
                    .imageScale(.small)
                    .foregroundStyle(bookmark.isTrackingRemote ? .secondary : .tertiary)
                Spacer(minLength: 8)
            }
            if let caption {
                Text(caption)
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
        .padding(.horizontal, 14)
    }
}
