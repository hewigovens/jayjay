import SwiftUI

extension RepoContentView {
    func changePaletteItems(selection: String) -> [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        let short = String(selection.prefix(8))
        // Safe actions — show selected change ID so user knows the target
        items.append(CommandPaletteItem(
            title: "New Child Change (\(short))",
            icon: "plus.circle",
            category: "Change"
        ) { viewModel.newChange(parent: selection) })
        if viewModel.change(for: selection)?.isImmutable != true {
            items.append(CommandPaletteItem(
                title: "Edit / Switch To (\(short))",
                icon: "pencil.circle",
                category: "Change"
            ) { viewModel.edit(rev: selection) })
            items.append(CommandPaletteItem(
                title: "Run Formatters on \(short) (jj fix)",
                icon: "wand.and.stars",
                category: "Change",
                keywords: ["format", "formatter", "fix", "lint"]
            ) { viewModel.fix(revs: [selection]) })
        }
        items.append(CommandPaletteItem(
            title: "Duplicate (\(short))",
            icon: "doc.on.doc",
            category: "Change"
        ) { viewModel.duplicate(rev: selection) })
        items.append(CommandPaletteItem(
            title: "Revert Change (\(short))",
            icon: "arrow.uturn.backward",
            category: "Change"
        ) { viewModel.revertChange(rev: selection) })
        items.append(CommandPaletteItem(
            title: "Create Bookmark on \(short)",
            icon: "bookmark",
            category: "Change"
        ) {
            presentBookmarkCreate(rev: selection)
        })
        return items
    }

    var multiSelectionPaletteItems: [CommandPaletteItem] {
        let revisions = viewModel.selectedChangeIds
        guard revisions.count > 1 else {
            return []
        }
        var items: [CommandPaletteItem] = []
        if viewModel.selectionCapabilities.canParallelize {
            items.append(CommandPaletteItem(
                title: "Parallelize \(revisions.count) selected",
                icon: "arrow.triangle.branch",
                category: "Change",
                keywords: ["parallel", "independent", "sibling", "split", "stack"]
            ) {
                viewModel.parallelize(revs: revisions)
            })
        }
        if viewModel.selectionCapabilities.canFix {
            items.append(CommandPaletteItem(
                title: "Run formatters on \(revisions.count) selected (jj fix)",
                icon: "wand.and.stars",
                category: "Change",
                keywords: ["format", "formatter", "fix", "lint", "stack"]
            ) {
                viewModel.fix(revs: revisions)
            })
        }
        return items
    }
}
