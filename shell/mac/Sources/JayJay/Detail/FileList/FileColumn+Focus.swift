import AppKit

extension ChangeDetailView {
    func focusFileList() {
        NSApp.keyWindow?.makeFirstResponder(nil)
        activePane = .fileColumn
        if !filteredDiff.contains(where: { $0.path == selectedPath }), let first = filteredDiff.first {
            selectSingleFile(first.path)
        }
    }

    func toggleFileFilter() {
        if showFileFilter {
            dismissFileFilter()
        } else {
            showFileFilter = true
        }
    }

    func dismissFileFilter() {
        fileFilter = ""
        showFileFilter = false
        focusFileList()
    }
}
