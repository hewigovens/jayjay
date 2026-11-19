import AppKit

extension ChangeDetailView {
    func focusFileList() {
        NSApp.keyWindow?.makeFirstResponder(nil)
        activePane = .fileColumn
        reconcileFileSelection()
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
