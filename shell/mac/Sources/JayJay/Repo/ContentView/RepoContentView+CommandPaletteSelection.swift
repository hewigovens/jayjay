import JayJayCore
import SwiftUI

extension RepoContentView {
    func changePaletteItems(selection: String) -> [CommandPaletteItem] {
        var items: [CommandPaletteItem] = []
        let short = String(selection.prefix(8))
        // Safe actions — show selected change ID so user knows the target
        items.append(CommandPaletteItem(
            title: Localized.Palette.newChildChange(change: short),
            icon: "plus.circle",
            category: Localized.Repo.change
        ) { viewModel.newChange(parent: selection) })
        if viewModel.change(for: selection)?.isImmutable != true {
            items.append(CommandPaletteItem(
                title: Localized.Palette.editSwitchTo(change: short),
                icon: "pencil.circle",
                category: Localized.Repo.change
            ) { viewModel.edit(rev: selection) })
            items.append(CommandPaletteItem(
                title: Localized.Palette.runFormattersOn(change: short),
                icon: "wand.and.stars",
                category: Localized.Repo.change,
                keywords: ["format", "formatter", "fix", "lint"]
            ) { viewModel.fix(revs: [selection]) })
        }
        items.append(CommandPaletteItem(
            title: Localized.Palette.duplicateChange(change: short),
            icon: "doc.on.doc",
            category: Localized.Repo.change
        ) { viewModel.duplicate(rev: selection) })
        items.append(CommandPaletteItem(
            title: Localized.Palette.revertChange(change: short),
            icon: "arrow.uturn.backward",
            category: Localized.Repo.change
        ) { viewModel.revertChange(rev: selection) })
        items.append(CommandPaletteItem(
            title: Localized.Palette.showEvolution(change: short),
            icon: "clock.arrow.circlepath",
            category: Localized.Repo.change,
            keywords: ["evolog", "obslog", "history"]
        ) { viewModel.showEvolog(rev: selection) })
        items.append(CommandPaletteItem(
            title: Localized.Palette.createBookmarkOn(change: short),
            icon: "bookmark",
            category: Localized.Repo.change
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
                title: Localized.Menu.parallelizeSelected(count: revisions.count),
                icon: "arrow.triangle.branch",
                category: Localized.Repo.change,
                keywords: ["parallel", "independent", "sibling", "split", "stack"]
            ) {
                viewModel.parallelize(revs: revisions)
            })
        }
        if viewModel.selectionCapabilities.canFix {
            items.append(CommandPaletteItem(
                title: Localized.Menu.fixSelected(count: revisions.count),
                icon: "wand.and.stars",
                category: Localized.Repo.change,
                keywords: ["format", "formatter", "fix", "lint", "stack"]
            ) {
                viewModel.fix(revs: revisions)
            })
        }
        return items
    }
}
