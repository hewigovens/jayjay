use std::collections::HashMap;
use std::sync::Arc;

use jayjay_core::dag::{
    self, ChangeCommand, DagLayout, OrderedSelection, SelectionClick, SelectionCommand,
    SelectionGraph, SelectionState,
};
use jayjay_core::{ChangeInfo, GraphEntry};

#[derive(uniffi::Record, Debug, Clone)]
pub struct DagLayoutData {
    lanes: HashMap<String, u32>,
    active_lane_indices_per_row: Vec<Vec<u32>>,
    pass_through_lane_indices_per_row: Vec<Vec<u32>>,
    missing_ancestry_rows: Vec<bool>,
    overflow_rows: Vec<bool>,
    display_lane_count: u32,
    compact: bool,
}

/// Everything the shell needs from one load; sending the entries back into Rust would copy every string again.
#[derive(uniffi::Record, Debug, Clone)]
pub struct GraphWithLayout {
    pub entries: Vec<GraphEntry>,
    pub layout: DagLayoutData,
    pub selection: Arc<DagSelectionGraph>,
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct DagLaneSpan {
    pub lane: u32,
    pub first_row: u32,
    pub last_row: u32,
}

#[derive(uniffi::Object, Debug)]
pub struct DagSelectionGraph(SelectionGraph);

impl DagSelectionGraph {
    pub(crate) fn from_entries(entries: &[GraphEntry], layout: &DagLayout) -> Self {
        Self(SelectionGraph::new(entries, layout))
    }
}

#[uniffi::export]
impl DagSelectionGraph {
    /// Only for a graph the shell patched itself; a loaded graph arrives with one already built.
    #[uniffi::constructor]
    pub fn new(entries: Vec<GraphEntry>) -> Self {
        Self::from_entries(&entries, &DagLayout::compute(&entries))
    }

    pub fn selection_state(&self, selected_commit_ids: Vec<String>) -> SelectionState {
        self.0.state(&selected_commit_ids)
    }

    pub fn lane_parent(&self, row: u32) -> Option<u32> {
        self.0.lane_parent(row as usize).map(|ix| ix as u32)
    }

    pub fn lane_child(&self, row: u32) -> Option<u32> {
        self.0.lane_child(row as usize).map(|ix| ix as u32)
    }

    pub fn lane_left(&self, row: u32) -> Option<u32> {
        self.0.lane_left(row as usize).map(|ix| ix as u32)
    }

    pub fn lane_right(&self, row: u32) -> Option<u32> {
        self.0.lane_right(row as usize).map(|ix| ix as u32)
    }

    pub fn lane_span(&self, row: u32) -> Option<DagLaneSpan> {
        self.0.lane_span(row as usize).map(|span| DagLaneSpan {
            lane: span.lane as u32,
            first_row: span.first_row as u32,
            last_row: span.last_row as u32,
        })
    }
}

#[uniffi::remote(Enum)]
pub enum ChangeCommand {
    NewChange,
    NewChangeBefore,
    NewChangeAfter,
    Edit,
    Describe,
    SquashIntoParent,
    MoveChangesToWorkingCopy,
    RebaseOntoTrunk,
    Fix,
    CreateBookmark,
    CreateTag,
    ShowEvolution,
    Duplicate,
    Absorb,
    Revert,
    Abandon,
}

#[uniffi::remote(Enum)]
pub enum SelectionCommand {
    Merge,
    Squash,
    Parallelize,
    Fix,
    RebaseOntoTrunk,
    Abandon,
}

#[uniffi::export]
fn change_commands() -> Vec<ChangeCommand> {
    ChangeCommand::ALL.to_vec()
}

#[uniffi::export]
fn available_change_commands(change: ChangeInfo) -> Vec<ChangeCommand> {
    ChangeCommand::available(&change)
}

#[uniffi::export]
fn change_command_title(command: ChangeCommand) -> String {
    command.title()
}

#[uniffi::export]
fn change_command_accelerator(command: ChangeCommand) -> Option<String> {
    command.accelerator().map(String::from)
}

#[uniffi::export]
fn change_command_shortcut_id(command: ChangeCommand) -> Option<String> {
    command.shortcut().map(|shortcut| shortcut.id().to_owned())
}

#[uniffi::export]
fn selection_commands() -> Vec<SelectionCommand> {
    SelectionCommand::ALL.to_vec()
}

#[uniffi::export]
fn selection_command_title(command: SelectionCommand, count: u32) -> String {
    command.title(count as usize)
}

#[uniffi::export]
fn selection_command_accelerator(command: SelectionCommand) -> Option<String> {
    command.accelerator().map(String::from)
}

#[uniffi::export]
fn selection_command_help(command: SelectionCommand, available: bool) -> String {
    command.help(available)
}

#[uniffi::export]
fn compute_dag_layout(entries: Vec<GraphEntry>) -> DagLayoutData {
    layout_data(DagLayout::compute(&entries))
}

#[uniffi::export]
fn ordered_selection(
    selected: Vec<String>,
    primary: Option<String>,
    anchor: Option<String>,
) -> OrderedSelection {
    OrderedSelection::new(selected, primary, anchor)
}

#[uniffi::export]
fn apply_selection_click(
    selection: OrderedSelection,
    click: SelectionClick,
    id: String,
    order: Vec<String>,
) -> OrderedSelection {
    let mut selection = selection;
    selection.apply(click, id, &order);
    selection
}

#[uniffi::export]
fn apply_pair_selection_click(
    selection: OrderedSelection,
    click: SelectionClick,
    id: String,
    order: Vec<String>,
) -> OrderedSelection {
    let mut selection = selection;
    selection.apply_pair(click, id, &order);
    selection
}

#[uniffi::export]
fn ordered_selection_ids(selection: OrderedSelection, order: Vec<String>) -> Vec<String> {
    selection.ordered(&order)
}

#[uniffi::export]
fn selection_is_contiguous(selection: OrderedSelection, order: Vec<String>) -> bool {
    selection.is_contiguous_in(&order)
}

pub(crate) fn layout_data(layout: DagLayout) -> DagLayoutData {
    let display_lane_count = layout.display_lane_count();
    let compact = layout.is_compact();
    DagLayoutData {
        lanes: layout
            .lanes
            .into_iter()
            .map(|(k, v)| (k, v as u32))
            .collect(),
        active_lane_indices_per_row: layout
            .active_lane_indices_per_row
            .iter()
            .map(|row| row.iter().map(|&v| v as u32).collect())
            .collect(),
        pass_through_lane_indices_per_row: layout
            .pass_through_lane_indices_per_row
            .iter()
            .map(|row| row.iter().map(|&v| v as u32).collect())
            .collect(),
        missing_ancestry_rows: layout.missing_ancestry_rows,
        overflow_rows: layout.overflow_rows.to_vec(),
        display_lane_count: display_lane_count as u32,
        compact,
    }
}

#[uniffi::export]
fn descendant_commit_ids(entries: Vec<GraphEntry>, commit_id: &str) -> Vec<String> {
    dag::descendant_commit_ids(&entries, commit_id)
}

#[uniffi::export]
fn can_rebase_onto(
    entries: Vec<GraphEntry>,
    source_commit_id: &str,
    target_commit_id: &str,
) -> bool {
    dag::can_rebase_onto(&entries, source_commit_id, target_commit_id)
}

#[uniffi::export]
fn merge_parent_choices(changes: Vec<jayjay_core::ChangeInfo>) -> Vec<dag::MergeParentChoice> {
    dag::MergeParentChoice::for_selection(&changes)
}
