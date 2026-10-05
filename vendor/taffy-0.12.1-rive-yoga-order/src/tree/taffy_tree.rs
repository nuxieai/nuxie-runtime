//! Contains [TaffyTree](crate::tree::TaffyTree): the default implementation of [LayoutTree](crate::tree::LayoutTree), and the error type for Taffy.
#[cfg(not(feature = "std"))]
use slotmap::SecondaryMap;
#[cfg(feature = "std")]
use slotmap::SparseSecondaryMap as SecondaryMap;
use slotmap::{DefaultKey, SlotMap};

#[cfg(feature = "block_layout")]
use crate::block::BlockContext;
use crate::geometry::Size;
use crate::style::{AvailableSpace, Display, Style};
use crate::sys::DefaultCheapStr;
use crate::tree::{
    Cache, ClearState, Layout, LayoutInput, LayoutOutput, LayoutPartialTree, NodeId, PrintTree, RoundTree, RunMode,
    TraversePartialTree, TraverseTree,
};
use crate::util::debug::{debug_log, debug_log_node};
use crate::util::sys::{new_vec_with_capacity, ChildrenVec, Vec};

use crate::compute::{compute_cached_layout, compute_hidden_layout, compute_root_layout, round_layout};
use crate::tree::rive_measure_cache::RiveMeasureCache;
use crate::tree::RiveMeasureMetadata;
use crate::CacheTree;

#[cfg(feature = "block_layout")]
use crate::{compute::compute_block_layout, LayoutBlockContainer};
#[cfg(feature = "flexbox")]
use crate::{compute::compute_flexbox_layout, LayoutFlexboxContainer};
#[cfg(feature = "grid")]
use crate::{compute::compute_grid_layout, LayoutGridContainer};

#[cfg(all(feature = "detailed_layout_info", feature = "grid"))]
use crate::compute::grid::DetailedGridInfo;
#[cfg(feature = "detailed_layout_info")]
use crate::tree::layout::DetailedLayoutInfo;

/// The error Taffy generates on invalid operations
pub type TaffyResult<T> = Result<T, TaffyError>;

/// An error that occurs while trying to access or modify a node's children by index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaffyError {
    /// The parent node does not have a child at `child_index`. It only has `child_count` children
    ChildIndexOutOfBounds {
        /// The parent node whose child was being looked up
        parent: NodeId,
        /// The index that was looked up
        child_index: usize,
        /// The total number of children the parent has
        child_count: usize,
    },
    /// The parent node was not found in the [`TaffyTree`](crate::TaffyTree) instance.
    InvalidParentNode(NodeId),
    /// The child node was not found in the [`TaffyTree`](crate::TaffyTree) instance.
    InvalidChildNode(NodeId),
    /// The supplied node was not found in the [`TaffyTree`](crate::TaffyTree) instance.
    InvalidInputNode(NodeId),
}

impl core::fmt::Display for TaffyError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            TaffyError::ChildIndexOutOfBounds { parent, child_index, child_count } => {
                write!(f, "Index (is {child_index}) should be < child_count ({child_count}) for parent node {parent:?}")
            }
            TaffyError::InvalidParentNode(parent) => {
                write!(f, "Parent Node {parent:?} is not in the TaffyTree instance")
            }
            TaffyError::InvalidChildNode(child) => {
                write!(f, "Child Node {child:?} is not in the TaffyTree instance")
            }
            TaffyError::InvalidInputNode(node) => {
                write!(f, "Supplied Node {node:?} is not in the TaffyTree instance")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TaffyError {}

/// Global configuration values for a TaffyTree instance
#[derive(Debug, Clone, Copy)]
pub(crate) struct TaffyConfig {
    /// Whether to round layout values
    pub(crate) use_rounding: bool,
}

impl Default for TaffyConfig {
    fn default() -> Self {
        Self { use_rounding: true }
    }
}

/// Layout information for a given [`Node`](crate::node::Node)
///
/// Stored in a [`TaffyTree`].
#[derive(Debug, Clone, PartialEq)]
struct NodeData {
    #[cfg(feature = "grid")]
    grid_virtual_rows: Vec<f32>,
    #[cfg(feature = "grid")]
    grid_virtual_columns: Vec<f32>,
    #[cfg(feature = "grid")]
    grid_virtual_sizings: Vec<(Style, Vec<u64>, Vec<f32>)>,
    /// The layout strategy used by this node
    pub(crate) style: Style,

    /// The always unrounded results of the layout computation. We must store this separately from the rounded
    /// layout to avoid errors from rounding already-rounded values. See <https://github.com/DioxusLabs/taffy/issues/501>.
    pub(crate) unrounded_layout: Layout,

    /// The final results of the layout computation.
    /// These may be rounded or unrounded depending on what the `use_rounding` config setting is set to.
    pub(crate) final_layout: Layout,

    /// Whether the node has context data associated with it or not
    pub(crate) has_context: bool,

    /// The cached results of the layout computation
    pub(crate) cache: Cache,

    // Rive: callbacks may distinguish an intrinsic probe from an ordinary
    // zero-space offer even when their final LayoutInput is identical.
    cache_measure_context: u8,
    cache_undefined_available: Size<Option<f32>>,
    rive_measure_cache: RiveMeasureCache,

    /// The computation result from layout algorithm
    #[cfg(feature = "detailed_layout_info")]
    pub(crate) detailed_layout_info: DetailedLayoutInfo,
}

impl NodeData {
    /// Create the data for a new node
    #[must_use]
    pub const fn new(style: Style) -> Self {
        Self {
            #[cfg(feature = "grid")]
            grid_virtual_rows: Vec::new(),
            #[cfg(feature = "grid")]
            grid_virtual_columns: Vec::new(),
            #[cfg(feature = "grid")]
            grid_virtual_sizings: Vec::new(),
            style,
            cache: Cache::new(),
            cache_measure_context: 0,
            cache_undefined_available: Size::NONE,
            rive_measure_cache: RiveMeasureCache::new(),
            unrounded_layout: Layout::new(),
            final_layout: Layout::new(),
            has_context: false,
            #[cfg(feature = "detailed_layout_info")]
            detailed_layout_info: DetailedLayoutInfo::None,
        }
    }

    /// Marks a node and all of its ancestors as requiring relayout
    ///
    /// This clears any cached data and signals that the data must be recomputed.
    /// If the node was already marked as dirty, returns true
    #[inline]
    pub fn mark_dirty(&mut self) -> ClearState {
        let generic = self.cache.clear();
        if self.rive_measure_cache.mark_dirty() {
            ClearState::Cleared
        } else {
            generic
        }
    }

    fn cached_layout(&self, input: &LayoutInput, context: u8) -> Option<LayoutOutput> {
        if self.cache_measure_context != context || self.cache_undefined_available != input.rive_undefined_available {
            return None;
        }
        self.cache.get(input)
    }

    fn store_layout(&mut self, input: &LayoutInput, output: LayoutOutput, context: u8) {
        if self.cache_undefined_available != input.rive_undefined_available {
            self.cache.clear();
            self.cache_undefined_available = input.rive_undefined_available;
        }
        if self.cache_measure_context != context {
            self.cache.clear();
            self.rive_measure_cache.clear();
            self.cache_measure_context = context;
        }
        self.cache.store(input, output);
    }
}

/// An entire tree of UI nodes. The entry point to Taffy's high-level API.
///
/// Allows you to build a tree of UI nodes, run Taffy's layout algorithms over that tree, and then access the resultant layout.]
#[derive(Debug, Clone)]
pub struct TaffyTree<NodeContext = ()> {
    /// The [`NodeData`] for each node stored in this tree
    nodes: SlotMap<DefaultKey, NodeData>,

    /// Functions/closures that compute the intrinsic size of leaf nodes
    node_context_data: SecondaryMap<DefaultKey, NodeContext>,

    /// The children of each node
    ///
    /// The indexes in the outer vector correspond to the position of the parent [`NodeData`]
    children: SlotMap<DefaultKey, ChildrenVec<NodeId>>,

    /// The parents of each node
    ///
    /// The indexes in the outer vector correspond to the position of the child [`NodeData`]
    parents: SlotMap<DefaultKey, Option<NodeId>>,

    /// Layout mode configuration
    config: TaffyConfig,
    rive_generation: u64,
}

impl Default for TaffyTree {
    fn default() -> TaffyTree<()> {
        TaffyTree::new()
    }
}

/// Iterator that wraps a slice of nodes, lazily converting them to u64
pub struct TaffyTreeChildIter<'a>(core::slice::Iter<'a, NodeId>);
impl Iterator for TaffyTreeChildIter<'_> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}

// TraversePartialTree impl for TaffyTree
impl<NodeContext> TraversePartialTree for TaffyTree<NodeContext> {
    type ChildIter<'a>
        = TaffyTreeChildIter<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn child_ids(&self, parent_node_id: NodeId) -> Self::ChildIter<'_> {
        TaffyTreeChildIter(self.children[parent_node_id.into()].iter())
    }

    #[inline(always)]
    fn child_count(&self, parent_node_id: NodeId) -> usize {
        self.children[parent_node_id.into()].len()
    }

    #[inline(always)]
    fn get_child_id(&self, parent_node_id: NodeId, id: usize) -> NodeId {
        self.children[parent_node_id.into()][id]
    }
}

// TraverseTree impl for TaffyTree
impl<NodeContext> TraverseTree for TaffyTree<NodeContext> {}

// CacheTree impl for TaffyTree
impl<NodeContext> CacheTree for TaffyTree<NodeContext> {
    fn cache_get(&self, node_id: NodeId, input: &LayoutInput) -> Option<LayoutOutput> {
        self.nodes[node_id.into()].cached_layout(input, 0)
    }

    fn cache_store(&mut self, node_id: NodeId, input: &LayoutInput, layout_output: LayoutOutput) {
        self.nodes[node_id.into()].store_layout(input, layout_output, 0)
    }

    fn cache_clear(&mut self, node_id: NodeId) {
        self.nodes[node_id.into()].cache.clear();
    }
}

// PrintTree impl for TaffyTree
impl<NodeContext> PrintTree for TaffyTree<NodeContext> {
    #[inline(always)]
    fn get_debug_label(&self, node_id: NodeId) -> &'static str {
        let node = &self.nodes[node_id.into()];
        let display = node.style.display;
        let num_children = self.child_count(node_id);

        match (num_children, display) {
            (_, Display::None) => "NONE",
            (0, _) => "LEAF",
            #[cfg(feature = "block_layout")]
            (_, Display::Block) => "BLOCK",
            #[cfg(feature = "flexbox")]
            (_, Display::Flex) => {
                use crate::FlexDirection;
                match node.style.flex_direction {
                    FlexDirection::Row | FlexDirection::RowReverse => "FLEX ROW",
                    FlexDirection::Column | FlexDirection::ColumnReverse => "FLEX COL",
                }
            }
            #[cfg(feature = "grid")]
            (_, Display::Grid) => "GRID",
        }
    }

    #[inline(always)]
    fn get_final_layout(&self, node_id: NodeId) -> Layout {
        if self.config.use_rounding {
            self.nodes[node_id.into()].final_layout
        } else {
            self.nodes[node_id.into()].unrounded_layout
        }
    }
}

/// View over the Taffy tree that holds the tree itself along with a reference to the context
/// and implements LayoutTree. This allows the context to be stored outside of the TaffyTree struct
/// which makes the lifetimes of the context much more flexible.
pub(crate) struct TaffyView<'t, NodeContext, MeasureFunction>
where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
{
    /// A reference to the TaffyTree
    pub(crate) taffy: &'t mut TaffyTree<NodeContext>,
    /// The context provided for passing to measure functions if layout is run over this struct
    pub(crate) measure_function: MeasureFunction,
    // Local to this solve, preserving nested probe scope without global state.
    min_content_probe_depth: Option<&'t core::cell::Cell<usize>>,
    rive_metadata: Option<&'t mut dyn FnMut(NodeId, Option<&NodeContext>, bool) -> RiveMeasureMetadata>,
}

impl<NodeContext, MeasureFunction> TaffyView<'_, NodeContext, MeasureFunction>
where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
{
    fn measure_cache_context(&self) -> u8 {
        match self.min_content_probe_depth {
            None => 0,
            Some(depth) if depth.get() == 0 => 1,
            Some(_) => 2,
        }
    }

    #[inline(always)]
    /// Unified implementation that both `LayoutPartialTree::compute_child_layout`
    /// and `LayoutBlockContainer::compute_block_child_layout` delegate to.
    fn compute_child_layout(
        &mut self,
        node_id: NodeId,
        inputs: LayoutInput,
        #[cfg(feature = "block_layout")] block_ctx: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        // If RunMode is PerformHiddenLayout then this indicates that an ancestor node is `Display::None`
        // and thus that we should lay out this node using hidden layout regardless of it's own display style.
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            debug_log!("HIDDEN");
            return compute_hidden_layout(self, node_id);
        }

        let key = node_id.into();
        let data = &self.taffy.nodes[key];
        #[cfg(feature = "grid")]
        let virtual_grid = data.style.display == Display::Grid
            && (!data.grid_virtual_rows.is_empty() || !data.grid_virtual_columns.is_empty());
        #[cfg(not(feature = "grid"))]
        let virtual_grid = false;
        if self.rive_metadata.is_some()
            && data.has_context
            && self.child_count(node_id) == 0
            && data.style.display != Display::None
            && !crate::CoreStyle::is_block(&data.style)
            && !virtual_grid
        {
            let probing = self.min_content_probe_depth.is_some_and(|depth| depth.get() != 0);
            let metadata =
                self.rive_metadata.as_mut().unwrap()(node_id, self.taffy.node_context_data.get(key), probing);
            let data = &mut self.taffy.nodes[key];
            // Generic and Rive layouts must not consume each other's entries.
            if data.cache_measure_context == 0 {
                data.cache.clear();
                data.rive_measure_cache.clear();
                data.cache_measure_context = 1;
            }
            let context = self.taffy.node_context_data.get_mut(key);
            let style = &data.style;
            let measure = &mut self.measure_function;
            return crate::compute::leaf::compute_leaf_layout_cached(
                inputs,
                style,
                |_, _| 0.0,
                |known, available| measure(known, available, node_id, context, style),
                true,
                Some((&mut data.rive_measure_cache, metadata, self.taffy.rive_generation)),
            );
        }

        // We run the following wrapped in "compute_cached_layout", which will check the cache for an entry matching the node and inputs and:
        //   - Return that entry if exists
        //   - Else call the passed closure (below) to compute the result
        //
        // If there was no cache match and a new result needs to be computed then that result will be added to the cache
        compute_cached_layout(self, node_id, inputs, |tree, node_id, inputs| {
            let display_mode = tree.taffy.nodes[node_id.into()].style.display;
            let has_children = tree.child_count(node_id) > 0;
            #[cfg(feature = "grid")]
            let has_children = has_children
                || (display_mode == Display::Grid
                    && (!tree.taffy.nodes[node_id.into()].grid_virtual_rows.is_empty()
                        || !tree.taffy.nodes[node_id.into()].grid_virtual_columns.is_empty()));

            debug_log!(display_mode);
            debug_log_node!(inputs);

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (display_mode, has_children) {
                (Display::None, _) => compute_hidden_layout(tree, node_id),
                #[cfg(feature = "block_layout")]
                (Display::Block, true) => compute_block_layout(tree, node_id, inputs, block_ctx),
                #[cfg(feature = "flexbox")]
                (Display::Flex, true) => compute_flexbox_layout(tree, node_id, inputs),
                #[cfg(feature = "grid")]
                (Display::Grid, true) => compute_grid_layout(tree, node_id, inputs),
                (_, false) => {
                    let node_key = node_id.into();
                    let style = &tree.taffy.nodes[node_key].style;
                    let has_context = tree.taffy.nodes[node_key].has_context;
                    let rive_measured_leaf = has_context && tree.min_content_probe_depth.is_some();
                    let node_context = has_context.then(|| tree.taffy.node_context_data.get_mut(node_key)).flatten();
                    let measure_function = |known_dimensions, available_space| {
                        (tree.measure_function)(known_dimensions, available_space, node_id, node_context, style)
                    };
                    crate::compute::leaf::compute_leaf_layout_with_rive_measurement(
                        inputs,
                        style,
                        |_, _| 0.0,
                        measure_function,
                        rive_measured_leaf,
                    )
                }
            }
        })
    }
}

// TraversePartialTree impl for TaffyView
impl<NodeContext, MeasureFunction> TraversePartialTree for TaffyView<'_, NodeContext, MeasureFunction>
where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
{
    type ChildIter<'a>
        = TaffyTreeChildIter<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn child_ids(&self, parent_node_id: NodeId) -> Self::ChildIter<'_> {
        self.taffy.child_ids(parent_node_id)
    }

    #[inline(always)]
    fn child_count(&self, parent_node_id: NodeId) -> usize {
        self.taffy.child_count(parent_node_id)
    }

    #[inline(always)]
    fn get_child_id(&self, parent_node_id: NodeId, child_index: usize) -> NodeId {
        self.taffy.get_child_id(parent_node_id, child_index)
    }
}

// TraverseTree impl for TaffyView
impl<NodeContext, MeasureFunction> TraverseTree for TaffyView<'_, NodeContext, MeasureFunction> where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>
{
}

// LayoutPartialTree impl for TaffyView
impl<NodeContext, MeasureFunction> LayoutPartialTree for TaffyView<'_, NodeContext, MeasureFunction>
where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
{
    type CoreContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type CustomIdent = DefaultCheapStr;

    #[inline(always)]
    fn get_core_container_style(&self, node_id: NodeId) -> Self::CoreContainerStyle<'_> {
        &self.taffy.nodes[node_id.into()].style
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, node_id: NodeId, layout: &Layout) {
        self.taffy.nodes[node_id.into()].unrounded_layout = *layout;
    }

    #[inline(always)]
    fn resolve_calc_value(&self, _val: *const (), _basis: f32) -> f32 {
        0.0
    }

    #[inline(always)]
    fn compute_child_layout(&mut self, node_id: NodeId, inputs: LayoutInput) -> LayoutOutput {
        self.compute_child_layout(
            node_id,
            inputs,
            #[cfg(feature = "block_layout")]
            None,
        )
    }

    fn begin_min_content_probe(&mut self) {
        if let Some(depth) = self.min_content_probe_depth {
            depth.set(depth.get() + 1);
        }
    }

    fn uses_rive_layout(&self) -> bool {
        self.min_content_probe_depth.is_some()
    }

    #[cfg(feature = "flexbox")]
    fn rive_container_is_row(&self, node: NodeId) -> bool {
        self.taffy.nodes[node.into()].style.flex_direction.is_row()
    }

    fn end_min_content_probe(&mut self, owner: NodeId) {
        if let Some(depth) = self.min_content_probe_depth {
            depth.set(depth.get() - 1);
            // TrackSizing::measureItemMinContent invalidates the probed item,
            // not every descendant nor the eight measurement entries.
            self.taffy.nodes[owner.into()].rive_measure_cache.invalidate_final();
            self.taffy.nodes[owner.into()].cache.clear();
        }
    }
}

impl<NodeContext, MeasureFunction> CacheTree for TaffyView<'_, NodeContext, MeasureFunction>
where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
{
    fn cache_get(&self, node_id: NodeId, input: &LayoutInput) -> Option<LayoutOutput> {
        self.taffy.nodes[node_id.into()].cached_layout(input, self.measure_cache_context())
    }

    fn cache_store(&mut self, node_id: NodeId, input: &LayoutInput, layout_output: LayoutOutput) {
        let context = self.measure_cache_context();
        self.taffy.nodes[node_id.into()].store_layout(input, layout_output, context)
    }

    fn cache_clear(&mut self, node_id: NodeId) {
        self.taffy.nodes[node_id.into()].cache.clear();
        self.taffy.nodes[node_id.into()].rive_measure_cache.clear();
    }
}

#[cfg(feature = "block_layout")]
impl<NodeContext, MeasureFunction> LayoutBlockContainer for TaffyView<'_, NodeContext, MeasureFunction>
where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
{
    type BlockContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type BlockItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[inline(always)]
    fn get_block_container_style(&self, node_id: NodeId) -> Self::BlockContainerStyle<'_> {
        self.get_core_container_style(node_id)
    }

    #[inline(always)]
    fn get_block_child_style(&self, child_node_id: NodeId) -> Self::BlockItemStyle<'_> {
        self.get_core_container_style(child_node_id)
    }

    #[inline(always)]
    fn compute_block_child_layout(
        &mut self,
        node_id: NodeId,
        inputs: LayoutInput,
        block_ctx: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        self.compute_child_layout(node_id, inputs, block_ctx)
    }
}

#[cfg(feature = "flexbox")]
impl<NodeContext, MeasureFunction> LayoutFlexboxContainer for TaffyView<'_, NodeContext, MeasureFunction>
where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
{
    type FlexboxContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type FlexboxItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[inline(always)]
    fn get_flexbox_container_style(&self, node_id: NodeId) -> Self::FlexboxContainerStyle<'_> {
        &self.taffy.nodes[node_id.into()].style
    }

    #[inline(always)]
    fn get_flexbox_child_style(&self, child_node_id: NodeId) -> Self::FlexboxItemStyle<'_> {
        &self.taffy.nodes[child_node_id.into()].style
    }
}

#[cfg(feature = "grid")]
impl<NodeContext, MeasureFunction> LayoutGridContainer for TaffyView<'_, NodeContext, MeasureFunction>
where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
{
    type GridContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type GridItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[inline(always)]
    fn get_grid_container_style(&self, node_id: NodeId) -> Self::GridContainerStyle<'_> {
        &self.taffy.nodes[node_id.into()].style
    }

    #[inline(always)]
    fn get_grid_child_style(&self, child_node_id: NodeId) -> Self::GridItemStyle<'_> {
        &self.taffy.nodes[child_node_id.into()].style
    }
    fn grid_virtual_contributions(&self, node_id: NodeId, rows: bool) -> &[f32] {
        let data = &self.taffy.nodes[node_id.into()];
        if rows {
            &data.grid_virtual_rows
        } else {
            &data.grid_virtual_columns
        }
    }
    fn grid_virtual_sizing(&self, node_id: NodeId, key: &[u64]) -> Option<Vec<f32>> {
        let data = &self.taffy.nodes[node_id.into()];
        data.grid_virtual_sizings
            .iter()
            .find(|(style, old, _)| style == &data.style && old == key)
            .map(|(_, _, sizes)| sizes.clone())
    }
    fn store_grid_virtual_sizing(&mut self, node_id: NodeId, key: Vec<u64>, sizes: Vec<f32>) {
        let data = &mut self.taffy.nodes[node_id.into()];
        if data.grid_virtual_sizings.iter().filter(|(_, old, _)| old[0] == key[0]).count() >= 4 {
            let first = data.grid_virtual_sizings.iter().position(|(_, old, _)| old[0] == key[0]).unwrap();
            data.grid_virtual_sizings.remove(first);
        }
        data.grid_virtual_sizings.push((data.style.clone(), key, sizes));
    }

    #[inline(always)]
    #[cfg(feature = "detailed_layout_info")]
    fn set_detailed_grid_info(&mut self, node_id: NodeId, detailed_grid_info: DetailedGridInfo) {
        self.taffy.nodes[node_id.into()].detailed_layout_info = DetailedLayoutInfo::Grid(Box::new(detailed_grid_info));
    }
}

// RoundTree impl for TaffyView
impl<NodeContext, MeasureFunction> RoundTree for TaffyView<'_, NodeContext, MeasureFunction>
where
    MeasureFunction:
        FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
{
    #[inline(always)]
    fn get_unrounded_layout(&self, node: NodeId) -> Layout {
        self.taffy.nodes[node.into()].unrounded_layout
    }

    #[inline(always)]
    fn set_final_layout(&mut self, node_id: NodeId, layout: &Layout) {
        self.taffy.nodes[node_id.into()].final_layout = *layout;
    }
}

#[allow(clippy::iter_cloned_collect)] // due to no-std support, we need to use `iter_cloned` instead of `collect`
impl<NodeContext> TaffyTree<NodeContext> {
    /// Rive's YGNodeSetGridVirtualContributions: normalize nonfinite values,
    /// retain independent owned contents, and invalidate only on change.
    #[cfg(feature = "grid")]
    pub fn set_grid_virtual_contributions(&mut self, node: NodeId, rows: &[f32], columns: &[f32]) -> TaffyResult<bool> {
        let finite = |v: &f32| if v.is_finite() { *v } else { 0.0 };
        let data = &mut self.nodes[node.into()];
        if data.grid_virtual_rows.iter().copied().eq(rows.iter().map(finite))
            && data.grid_virtual_columns.iter().copied().eq(columns.iter().map(finite))
        {
            return Ok(false);
        }
        data.grid_virtual_rows = rows.iter().map(finite).collect();
        data.grid_virtual_columns = columns.iter().map(finite).collect();
        data.grid_virtual_sizings.clear();
        self.mark_dirty(node)?;
        Ok(true)
    }
    /// Creates a new [`TaffyTree`]
    ///
    /// The default capacity of a [`TaffyTree`] is 16 nodes.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Creates a new [`TaffyTree`] that can store `capacity` nodes before reallocation
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        TaffyTree {
            // TODO: make this method const upstream,
            // so constructors here can be const
            nodes: SlotMap::with_capacity(capacity),
            children: SlotMap::with_capacity(capacity),
            parents: SlotMap::with_capacity(capacity),
            node_context_data: SecondaryMap::with_capacity(capacity),
            config: TaffyConfig::default(),
            rive_generation: 0,
        }
    }

    /// Enable rounding of layout values. Rounding is enabled by default.
    pub fn enable_rounding(&mut self) {
        self.config.use_rounding = true;
    }

    /// Disable rounding of layout values. Rounding is enabled by default.
    pub fn disable_rounding(&mut self) {
        self.config.use_rounding = false;
    }

    /// Creates and adds a new unattached leaf node to the tree, and returns the node of the new node
    pub fn new_leaf(&mut self, layout: Style) -> TaffyResult<NodeId> {
        let id = self.nodes.insert(NodeData::new(layout));
        let _ = self.children.insert(new_vec_with_capacity(0));
        let _ = self.parents.insert(None);

        Ok(id.into())
    }

    /// Creates and adds a new unattached leaf node to the tree, and returns the [`NodeId`] of the new node
    ///
    /// Creates and adds a new leaf node with a supplied context
    pub fn new_leaf_with_context(&mut self, layout: Style, context: NodeContext) -> TaffyResult<NodeId> {
        let mut data = NodeData::new(layout);
        data.has_context = true;

        let id = self.nodes.insert(data);
        self.node_context_data.insert(id, context);

        let _ = self.children.insert(new_vec_with_capacity(0));
        let _ = self.parents.insert(None);

        Ok(id.into())
    }

    /// Creates and adds a new node, which may have any number of `children`
    pub fn new_with_children(&mut self, layout: Style, children: &[NodeId]) -> TaffyResult<NodeId> {
        let id = NodeId::from(self.nodes.insert(NodeData::new(layout)));

        for child in children {
            self.parents[(*child).into()] = Some(id);
        }

        let _ = self.children.insert(children.iter().copied().collect::<_>());
        let _ = self.parents.insert(None);

        Ok(id)
    }

    /// Drops all nodes in the tree
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.children.clear();
        self.parents.clear();
    }

    /// Remove a specific node from the tree and drop it
    ///
    /// Returns the id of the node removed.
    pub fn remove(&mut self, node: NodeId) -> TaffyResult<NodeId> {
        let key = node.into();
        if let Some(parent) = self.parents[key] {
            if let Some(children) = self.children.get_mut(parent.into()) {
                children.retain(|f| *f != node);
            }
        }

        // Remove "parent" references to a node when removing that node
        if let Some(children) = self.children.get(key) {
            for child in children.iter().copied() {
                self.parents[child.into()] = None;
            }
        }

        let _ = self.children.remove(key);
        let _ = self.parents.remove(key);
        let _ = self.nodes.remove(key);

        Ok(node)
    }

    /// Sets the context data associated with the node
    #[inline]
    pub fn set_node_context(&mut self, node: NodeId, measure: Option<NodeContext>) -> TaffyResult<()> {
        let key = node.into();
        if let Some(measure) = measure {
            self.nodes[key].has_context = true;
            self.node_context_data.insert(key, measure);
        } else {
            self.nodes[key].has_context = false;
            self.node_context_data.remove(key);
        }

        self.mark_dirty(node)?;

        Ok(())
    }

    /// Gets a reference to the the context data associated with the node
    #[inline]
    pub fn get_node_context(&self, node: NodeId) -> Option<&NodeContext> {
        self.node_context_data.get(node.into())
    }

    /// Gets a mutable reference to the the context data associated with the node
    #[inline]
    pub fn get_node_context_mut(&mut self, node: NodeId) -> Option<&mut NodeContext> {
        self.node_context_data.get_mut(node.into())
    }

    /// Gets mutable references to the the context data associated with the nodes. All keys must be valid and disjoint, otherwise None is returned.
    pub fn get_disjoint_node_context_mut<const N: usize>(
        &mut self,
        keys: [NodeId; N],
    ) -> Option<[&mut NodeContext; N]> {
        self.node_context_data.get_disjoint_mut(keys.map(|k| k.into()))
    }

    /// Adds a `child` node under the supplied `parent`
    pub fn add_child(&mut self, parent: NodeId, child: NodeId) -> TaffyResult<()> {
        let parent_key = parent.into();
        let child_key = child.into();
        self.parents[child_key] = Some(parent);
        self.children[parent_key].push(child);
        self.mark_dirty(parent)?;

        Ok(())
    }

    /// Inserts a `child` node at the given `child_index` under the supplied `parent`, shifting all children after it to the right.
    pub fn insert_child_at_index(&mut self, parent: NodeId, child_index: usize, child: NodeId) -> TaffyResult<()> {
        let parent_key = parent.into();

        let child_count = self.children[parent_key].len();
        if child_index > child_count {
            return Err(TaffyError::ChildIndexOutOfBounds { parent, child_index, child_count });
        }

        self.parents[child.into()] = Some(parent);
        self.children[parent_key].insert(child_index, child);
        self.mark_dirty(parent)?;

        Ok(())
    }

    /// Directly sets the `children` of the supplied `parent`
    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) -> TaffyResult<()> {
        let parent_key = parent.into();

        // Remove node as parent from all its current children.
        for child in &self.children[parent_key] {
            self.parents[(*child).into()] = None;
        }

        // Build up relation node <-> child
        for &child in children {
            // Remove child from previous parent
            if let Some(previous_parent) = self.parents[child.into()] {
                self.remove_child(previous_parent, child).unwrap();
            }
            self.parents[child.into()] = Some(parent);
        }

        let parent_children = &mut self.children[parent_key];
        parent_children.clear();
        children.iter().for_each(|child| parent_children.push(*child));

        self.mark_dirty(parent)?;

        Ok(())
    }

    /// Removes the `child` of the parent `node`
    ///
    /// The child is not removed from the tree entirely, it is simply no longer attached to its previous parent.
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) -> TaffyResult<NodeId> {
        let index = self.children[parent.into()].iter().position(|n| *n == child).unwrap();
        self.remove_child_at_index(parent, index)
    }

    /// Removes the child at the given `index` from the `parent`
    ///
    /// The child is not removed from the tree entirely, it is simply no longer attached to its previous parent.
    pub fn remove_child_at_index(&mut self, parent: NodeId, child_index: usize) -> TaffyResult<NodeId> {
        let parent_key = parent.into();
        let child_count = self.children[parent_key].len();
        if child_index >= child_count {
            return Err(TaffyError::ChildIndexOutOfBounds { parent, child_index, child_count });
        }

        let child = self.children[parent_key].remove(child_index);
        self.parents[child.into()] = None;

        self.mark_dirty(parent)?;

        Ok(child)
    }

    /// Removes children at the given range from the `parent`
    ///
    /// Children are not removed from the tree entirely, they are simply no longer attached to their previous parent.
    ///
    /// Function will panic if given range is invalid. See [`core::slice::range`]
    pub fn remove_children_range<R>(&mut self, parent: NodeId, range: R) -> TaffyResult<()>
    where
        R: core::ops::RangeBounds<usize>,
    {
        let parent_key = parent.into();
        for child in self.children[parent_key].drain(range) {
            self.parents[child.into()] = None;
        }

        self.mark_dirty(parent)?;
        Ok(())
    }

    /// Replaces the child at the given `child_index` from the `parent` node with the new `child` node
    ///
    /// The child is not removed from the tree entirely, it is simply no longer attached to its previous parent.
    pub fn replace_child_at_index(
        &mut self,
        parent: NodeId,
        child_index: usize,
        new_child: NodeId,
    ) -> TaffyResult<NodeId> {
        let parent_key = parent.into();

        let child_count = self.children[parent_key].len();
        if child_index >= child_count {
            return Err(TaffyError::ChildIndexOutOfBounds { parent, child_index, child_count });
        }

        self.parents[new_child.into()] = Some(parent);
        let old_child = core::mem::replace(&mut self.children[parent_key][child_index], new_child);
        self.parents[old_child.into()] = None;

        self.mark_dirty(parent)?;

        Ok(old_child)
    }

    /// Returns the child node of the parent `node` at the provided `child_index`
    #[inline]
    pub fn child_at_index(&self, parent: NodeId, child_index: usize) -> TaffyResult<NodeId> {
        let parent_key = parent.into();
        let child_count = self.children[parent_key].len();
        if child_index >= child_count {
            return Err(TaffyError::ChildIndexOutOfBounds { parent, child_index, child_count });
        }

        Ok(self.children[parent_key][child_index])
    }

    /// Returns the total number of nodes in the tree
    #[inline]
    pub fn total_node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the `NodeId` of the parent node of the specified node (if it exists)
    ///
    /// - Return None if the specified node has no parent
    /// - Panics if the specified node does not exist
    #[inline]
    pub fn parent(&self, child_id: NodeId) -> Option<NodeId> {
        self.parents[child_id.into()]
    }

    /// Returns a list of children that belong to the parent node
    pub fn children(&self, parent: NodeId) -> TaffyResult<Vec<NodeId>> {
        Ok(self.children[parent.into()].clone())
    }

    /// Sets the [`Style`] of the provided `node`
    #[inline]
    pub fn set_style(&mut self, node: NodeId, style: Style) -> TaffyResult<()> {
        self.nodes[node.into()].style = style;
        self.mark_dirty(node)?;
        Ok(())
    }

    /// Gets the [`Style`] of the provided `node`
    #[inline]
    pub fn style(&self, node: NodeId) -> TaffyResult<&Style> {
        Ok(&self.nodes[node.into()].style)
    }

    /// Return this node layout relative to its parent
    #[inline]
    pub fn layout(&self, node: NodeId) -> TaffyResult<&Layout> {
        if self.config.use_rounding {
            Ok(&self.nodes[node.into()].final_layout)
        } else {
            Ok(&self.nodes[node.into()].unrounded_layout)
        }
    }

    /// Returns this node layout with unrounded values relative to its parent.
    #[inline]
    pub fn unrounded_layout(&self, node: NodeId) -> &Layout {
        &self.nodes[node.into()].unrounded_layout
    }

    /// Get the "detailed layout info" for a node.
    ///
    /// Currently this is only implemented for CSS Grid containers where it contains
    /// the computed size of each grid track and the computed placement of each grid item
    #[cfg(feature = "detailed_layout_info")]
    #[inline]
    pub fn detailed_layout_info(&self, node_id: NodeId) -> &DetailedLayoutInfo {
        &self.nodes[node_id.into()].detailed_layout_info
    }

    /// Marks the layout of this node and its ancestors as outdated
    pub fn mark_dirty(&mut self, node: NodeId) -> TaffyResult<()> {
        fn mark_dirty_recursive(
            nodes: &mut SlotMap<DefaultKey, NodeData>,
            parents: &SlotMap<DefaultKey, Option<NodeId>>,
            node_key: DefaultKey,
        ) {
            match nodes[node_key].mark_dirty() {
                ClearState::AlreadyEmpty => {
                    // Node was already marked as dirty.
                    // No need to visit ancestors
                    // as they should be marked as dirty already.
                }
                ClearState::Cleared => {
                    if let Some(Some(node)) = parents.get(node_key) {
                        mark_dirty_recursive(nodes, parents, (*node).into());
                    }
                }
            }
        }

        mark_dirty_recursive(&mut self.nodes, &self.parents, node.into());

        Ok(())
    }

    /// Indicates whether the layout of this node needs to be recomputed
    #[inline]
    pub fn dirty(&self, node: NodeId) -> TaffyResult<bool> {
        Ok(self.nodes[node.into()].cache.is_empty())
    }

    /// Updates the stored layout of the provided `node` and its children
    pub fn compute_layout_with_measure<MeasureFunction>(
        &mut self,
        node_id: NodeId,
        available_space: Size<AvailableSpace>,
        measure_function: MeasureFunction,
    ) -> Result<(), TaffyError>
    where
        MeasureFunction:
            FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style) -> Size<f32>,
    {
        let use_rounding = self.config.use_rounding;
        let mut taffy_view =
            TaffyView { taffy: self, measure_function, min_content_probe_depth: None, rive_metadata: None };
        compute_root_layout(&mut taffy_view, node_id, available_space);
        if use_rounding {
            round_layout(&mut taffy_view, node_id);
        }
        Ok(())
    }

    /// Rive: measure with the same nested min-content probe context exposed by
    /// Yoga's YGConfigIsMeasuringMinContent. Probe and ordinary cache entries
    /// cannot alias, even if descendants receive identical constraints.
    /// Context-bearing non-block leaves also use Yoga's exact/exact shortcut:
    /// final layout does not request intrinsic content size for fixed axes.
    pub fn compute_layout_with_measure_and_probe<MeasureFunction, MetadataFunction>(
        &mut self,
        node_id: NodeId,
        available_space: Size<AvailableSpace>,
        mut metadata: MetadataFunction,
        mut measure_function: MeasureFunction,
    ) -> Result<(), TaffyError>
    where
        MetadataFunction: FnMut(NodeId, Option<&NodeContext>, bool) -> RiveMeasureMetadata,
        MeasureFunction:
            FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>, &Style, bool) -> Size<f32>,
    {
        let use_rounding = self.config.use_rounding;
        let depth = core::cell::Cell::new(0);
        self.rive_generation = self.rive_generation.wrapping_add(1);
        let mut taffy_view = TaffyView {
            taffy: self,
            measure_function: |known, available, node, context, style| {
                measure_function(known, available, node, context, style, depth.get() != 0)
            },
            min_content_probe_depth: Some(&depth),
            rive_metadata: Some(&mut metadata),
        };
        compute_root_layout(&mut taffy_view, node_id, available_space);
        if use_rounding {
            round_layout(&mut taffy_view, node_id);
        }
        Ok(())
    }

    /// Updates the stored layout of the provided `node` and its children
    pub fn compute_layout(&mut self, node: NodeId, available_space: Size<AvailableSpace>) -> Result<(), TaffyError> {
        self.compute_layout_with_measure(node, available_space, |_, _, _, _, _| Size::ZERO)
    }

    /// Prints a debug representation of the tree's layout
    #[cfg(feature = "std")]
    pub fn print_tree(&mut self, root: NodeId) {
        crate::util::print_tree(self, root)
    }

    /// Returns an instance of LayoutTree representing the TaffyTree
    #[cfg(test)]
    pub(crate) fn as_layout_tree(&mut self) -> impl LayoutPartialTree + CacheTree + '_ {
        TaffyView {
            taffy: self,
            measure_function: |_, _, _, _, _| Size::ZERO,
            min_content_probe_depth: None,
            rive_metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::style::{Dimension, Display, FlexDirection};
    use crate::style_helpers::*;
    use crate::util::sys;

    #[test]
    fn rive_exact_leaf_skips_measurement_without_changing_final_box() {
        fn run(style: Style, rive: bool, context: bool) -> (usize, Layout) {
            let mut tree = TaffyTree::<()>::new();
            let node =
                if context { tree.new_leaf_with_context(style, ()).unwrap() } else { tree.new_leaf(style).unwrap() };
            let mut calls = 0;
            if rive {
                tree.compute_layout_with_measure_and_probe(
                    node,
                    Size::MAX_CONTENT,
                    |_, _, _| RiveMeasureMetadata::default(),
                    |_, _, _, _, _, _| {
                        calls += 1;
                        Size { width: 33.0, height: 44.0 }
                    },
                )
                .unwrap();
            } else {
                tree.compute_layout_with_measure(node, Size::MAX_CONTENT, |_, _, _, _, _| {
                    calls += 1;
                    Size { width: 33.0, height: 44.0 }
                })
                .unwrap();
            }
            (calls, *tree.layout(node).unwrap())
        }

        let fixed = Style { size: Size { width: length(100.0), height: length(80.0) }, ..Style::default() };
        let bounded = Style {
            min_size: Size { width: auto(), height: length(90.0) },
            max_size: Size { width: length(60.0), height: auto() },
            padding: crate::geometry::Rect {
                left: length(7.0),
                right: length(7.0),
                top: length(7.0),
                bottom: length(7.0),
            },
            ..fixed.clone()
        };
        let aspect = Style {
            size: Size { width: length(100.0), height: length(10.0) },
            aspect_ratio: Some(2.0),
            ..Style::default()
        };
        let padding_floor = Style {
            size: Size { width: length(1.0), height: length(1.0) },
            padding: crate::geometry::Rect {
                left: length(10.0),
                right: length(10.0),
                top: length(10.0),
                bottom: length(10.0),
            },
            ..Style::default()
        };
        for style in [fixed.clone(), bounded, aspect, padding_floor] {
            let (generic_calls, generic) = run(style.clone(), false, true);
            let (rive_calls, rive) = run(style, true, true);
            assert!(generic_calls > 0);
            assert_eq!(rive_calls, 0);
            assert_eq!(rive.size, generic.size);
            assert_eq!(rive.location, generic.location);
            assert_eq!(rive.padding, generic.padding);
            assert_eq!(rive.border, generic.border);
        }

        // No measured context means no Rive shortcut; generic callbacks retain
        // their behavior. One definite axis or AtMost offers still need measure.
        assert!(run(fixed, true, false).0 > 0);
        assert!(
            run(Style { size: Size { width: length(100.0), height: auto() }, ..Style::default() }, true, true).0 > 0
        );
        assert!(
            run(
                Style { max_size: Size { width: length(100.0), height: length(80.0) }, ..Style::default() },
                true,
                true
            )
            .0 > 0
        );
        #[cfg(feature = "block_layout")]
        assert!(
            run(
                Style {
                    display: Display::Block,
                    size: Size { width: length(100.0), height: length(80.0) },
                    ..Style::default()
                },
                true,
                true
            )
            .0 > 0
        );
    }

    #[test]
    fn rive_exact_leaf_shortcut_does_not_leak_into_generic_cache() {
        let mut tree = TaffyTree::<()>::new();
        let node = tree
            .new_leaf_with_context(
                Style { size: Size { width: length(100.0), height: length(80.0) }, ..Style::default() },
                (),
            )
            .unwrap();
        for _ in 0..2 {
            let mut generic_calls = 0;
            tree.compute_layout_with_measure(node, Size::MAX_CONTENT, |_, _, _, _, _| {
                generic_calls += 1;
                Size { width: 33.0, height: 44.0 }
            })
            .unwrap();
            assert!(generic_calls > 0);
            let mut rive_calls = 0;
            tree.compute_layout_with_measure_and_probe(
                node,
                Size::MAX_CONTENT,
                |_, _, _| RiveMeasureMetadata::default(),
                |_, _, _, _, _, _| {
                    rive_calls += 1;
                    Size { width: 33.0, height: 44.0 }
                },
            )
            .unwrap();
            assert_eq!(rive_calls, 0);
        }
    }

    fn size_measure_function(
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _node_id: NodeId,
        node_context: Option<&mut Size<f32>>,
        _style: &Style,
    ) -> Size<f32> {
        known_dimensions.unwrap_or(node_context.cloned().unwrap_or(Size::ZERO))
    }

    #[test]
    fn rive_nested_probe_does_not_share_an_ordinary_zero_space_cache() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let node = taffy.new_leaf(Style::default()).unwrap();
        let depth = core::cell::Cell::new(0usize);
        let mut view = TaffyView {
            taffy: &mut taffy,
            measure_function: |_: Size<Option<f32>>,
                               _: Size<AvailableSpace>,
                               _: NodeId,
                               _: Option<&mut ()>,
                               _: &Style| {
                Size { width: if depth.get() > 0 { 80.0 } else { 0.0 }, height: 0.0 }
            },
            min_content_probe_depth: Some(&depth),
            rive_metadata: None,
        };
        let input = LayoutInput {
            run_mode: RunMode::ComputeSize,
            available_space: Size { width: AvailableSpace::Definite(0.0), height: AvailableSpace::Definite(0.0) },
            ..LayoutInput::HIDDEN
        };
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, node, input).size.width, 0.0);
        view.begin_min_content_probe();
        view.begin_min_content_probe();
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, node, input).size.width, 80.0);
        view.end_min_content_probe(node);
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, node, input).size.width, 80.0);
        view.end_min_content_probe(node);
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, node, input).size.width, 0.0);
    }

    #[test]
    fn rive_measured_probe_invalidates_only_its_owner_final_slot() {
        let mut tree = TaffyTree::<()>::new();
        let owner = tree.new_leaf_with_context(Style::default(), ()).unwrap();
        let other = tree.new_leaf_with_context(Style::default(), ()).unwrap();
        tree.rive_generation = 1;
        let depth = core::cell::Cell::new(0);
        let calls = core::cell::Cell::new(0);
        let mut metadata = |_, _: Option<&()>, _| RiveMeasureMetadata::default();
        let mut view = TaffyView {
            taffy: &mut tree,
            measure_function: |_, _, _, _: Option<&mut ()>, _: &Style| {
                calls.set(calls.get() + 1);
                Size { width: if depth.get() > 0 { 80.0 } else { 0.0 }, height: 0.0 }
            },
            min_content_probe_depth: Some(&depth),
            rive_metadata: Some(&mut metadata),
        };
        let input = LayoutInput {
            run_mode: RunMode::PerformLayout,
            available_space: Size { width: AvailableSpace::Definite(0.0), height: AvailableSpace::MaxContent },
            ..LayoutInput::HIDDEN
        };
        view.begin_min_content_probe();
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, owner, input).size.width, 80.0);
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, other, input).size.width, 80.0);
        view.end_min_content_probe(owner);
        // The pinned grid owner invalidation deliberately is not a subtree or
        // global probe-lane reset. The other node's final result stays cached.
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, other, input).size.width, 80.0);
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, owner, input).size.width, 0.0);
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn rive_stretch_cross_offer_respects_overflow_margins_aspect_and_parent_mode() {
        use crate::style::{FlexDirection, FlexWrap};
        use crate::style_helpers::{auto, percent};
        fn run(mut parent: Style, child: Style, hidden_sibling: bool, rive: bool) -> (usize, Size<f32>) {
            let mut tree = TaffyTree::<()>::new();
            tree.disable_rounding();
            let leaf = tree.new_leaf_with_context(child, ()).unwrap();
            let hidden = tree.new_leaf(Style { display: Display::None, ..Style::default() }).unwrap();
            let children = if hidden_sibling { vec![leaf, hidden] } else { vec![leaf] };
            parent.flex_direction = FlexDirection::Row;
            let root = tree.new_with_children(parent, &children).unwrap();
            let mut calls = 0;
            if rive {
                tree.compute_layout_with_measure_and_probe(
                    root,
                    Size::MAX_CONTENT,
                    |_, _, _| RiveMeasureMetadata::default(),
                    |known, _, _, _, _, _| {
                        calls += 1;
                        known.unwrap_or(Size { width: 20.0, height: 10.0 })
                    },
                )
                .unwrap();
            } else {
                tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |known, _, _, _, _| {
                    calls += 1;
                    known.unwrap_or(Size { width: 20.0, height: 10.0 })
                })
                .unwrap();
            }
            (calls, tree.layout(leaf).unwrap().size)
        }
        let parent = Style { size: Size { width: length(100.0), height: length(80.0) }, ..Style::default() };
        let child = Style {
            flex_basis: length(20.0),
            flex_shrink: 0.0,
            min_size: Size { width: length(0.0), height: length(0.0) },
            ..Style::default()
        };
        assert_eq!(run(parent.clone(), child.clone(), false, true), (0, Size { width: 20.0, height: 80.0 }));
        assert!(run(parent.clone(), child.clone(), false, false).0 > 0);
        let wrap = Style { flex_wrap: FlexWrap::Wrap, ..parent.clone() };
        assert_eq!(run(wrap.clone(), child.clone(), false, true).0, 0);
        assert!(run(wrap, Style { flex_basis: length(120.0), ..child.clone() }, false, true).0 > 0);
        assert!(
            run(
                parent.clone(),
                Style {
                    margin: crate::geometry::Rect { top: auto(), ..crate::geometry::Rect::zero() },
                    ..child.clone()
                },
                false,
                true
            )
            .0 > 0
        );
        assert!(run(parent.clone(), Style { aspect_ratio: Some(2.0), ..child.clone() }, false, true).0 > 0);
        assert!(
            run(
                Style { size: Size { width: length(100.0), height: auto() }, ..parent.clone() },
                child.clone(),
                false,
                true
            )
            .0 > 0
        );
        // Yoga counts the hidden sibling in the gap multiplier, and resolves
        // this percentage main gap against cross=200, not main=100: 90+20>100.
        assert!(
            run(
                Style {
                    size: Size { width: length(100.0), height: length(200.0) },
                    flex_wrap: FlexWrap::Wrap,
                    gap: Size { width: percent(0.1), height: length(0.0) },
                    ..parent
                },
                Style { flex_basis: length(90.0), ..child },
                true,
                true
            )
            .0 > 0
        );
    }

    #[test]
    fn rive_undefined_numeric_offer_preserves_percent_basis_and_cache_identity() {
        let mut tree = TaffyTree::<()>::new();
        tree.disable_rounding();
        let child = tree
            .new_leaf(Style {
                size: Size { width: percent(0.5), height: length(10.0) },
                flex_shrink: 0.0,
                ..Style::default()
            })
            .unwrap();
        let root = tree
            .new_with_children(Style { flex_direction: FlexDirection::Column, ..Style::default() }, &[child])
            .unwrap();
        let depth = core::cell::Cell::new(0);
        let mut metadata = |_, _: Option<&()>, _| RiveMeasureMetadata::default();
        let mut view = TaffyView {
            taffy: &mut tree,
            min_content_probe_depth: Some(&depth),
            rive_metadata: Some(&mut metadata),
            measure_function: |_, _, _, _: Option<&mut ()>, _: &Style| Size::ZERO,
        };
        // A finite Undefined offer is neither the containing owner's1000 nor
        // a fixed/AtMost own width. Changing/leaving/re-entering it cannot hit
        // a generic cache entry computed with another percentage basis.
        for (offer, expected) in [(Some(400.0), 200.0), (Some(600.0), 300.0), (None, 0.0), (Some(400.0), 200.0)] {
            let output = LayoutPartialTree::compute_child_layout(
                &mut view,
                root,
                LayoutInput {
                    run_mode: RunMode::PerformLayout,
                    parent_size: Size { width: Some(1000.0), height: None },
                    rive_undefined_available: Size { width: offer, height: None },
                    ..LayoutInput::HIDDEN
                },
            );
            assert_eq!(output.size.width, expected);
            assert_eq!(view.taffy.nodes[child.into()].unrounded_layout.size.width, expected);
        }
    }

    #[test]
    fn rive_undefined_offer_applies_insets_maximum_and_measured_mode() {
        let mut tree = TaffyTree::<()>::new();
        let measured = tree.new_leaf_with_context(Style::default(), ()).unwrap();
        let child = tree
            .new_leaf(Style {
                size: Size { width: percent(0.5), height: length(10.0) },
                flex_shrink: 0.0,
                ..Style::default()
            })
            .unwrap();
        let root = tree
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Column,
                    max_size: Size { width: length(300.0), height: auto() },
                    padding: crate::geometry::Rect {
                        left: percent(0.1),
                        right: percent(0.1),
                        top: length(0.0),
                        bottom: length(0.0),
                    },
                    ..Style::default()
                },
                &[child],
            )
            .unwrap();
        let depth = core::cell::Cell::new(0);
        let calls = core::cell::Cell::new(0);
        let mut metadata = |_, _: Option<&()>, _| RiveMeasureMetadata::default();
        let mut view = TaffyView {
            taffy: &mut tree,
            min_content_probe_depth: Some(&depth),
            rive_metadata: Some(&mut metadata),
            measure_function: |known: Size<Option<f32>>, available, _, context: Option<&mut ()>, _: &Style| {
                if context.is_none() {
                    return known.unwrap_or(Size::ZERO);
                }
                calls.set(calls.get() + 1);
                assert_eq!(known.width, None);
                assert_eq!(available.width, AvailableSpace::MaxContent);
                Size { width: 350.0, height: 10.0 }
            },
        };
        let input = LayoutInput {
            run_mode: RunMode::PerformLayout,
            parent_size: Size { width: Some(1000.0), height: None },
            rive_undefined_available: Size { width: Some(400.0), height: None },
            ..LayoutInput::HIDDEN
        };
        // Own padding resolves against1000 (200 total); numeric inner offer
        // is capped by max300 minus padding, so the50% descendant gets50.
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, root, input).size.width, 250.0);
        assert_eq!(view.taffy.nodes[child.into()].unrounded_layout.size.width, 50.0);
        // The numeric offer must not reach a measured callback as AtMost.
        assert_eq!(LayoutPartialTree::compute_child_layout(&mut view, measured, input).size.width, 350.0);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn rive_undefined_container_forwards_numeric_atmost_to_measured_descendant() {
        let mut tree = TaffyTree::<()>::new();
        let child = tree
            .new_leaf_with_context(
                Style { min_size: Size { width: length(0.0), height: length(0.0) }, ..Style::default() },
                (),
            )
            .unwrap();
        let root = tree
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Column,
                    align_items: Some(crate::style::AlignItems::FLEX_START),
                    ..Style::default()
                },
                &[child],
            )
            .unwrap();
        let depth = core::cell::Cell::new(0);
        let observed = core::cell::Cell::new(false);
        let mut metadata = |_, _: Option<&()>, _| RiveMeasureMetadata::default();
        let mut view = TaffyView {
            taffy: &mut tree,
            min_content_probe_depth: Some(&depth),
            rive_metadata: Some(&mut metadata),
            measure_function: |known: Size<Option<f32>>,
                               available: Size<AvailableSpace>,
                               _,
                               _: Option<&mut ()>,
                               _: &Style| {
                if known.width.is_none() && available.width == AvailableSpace::Definite(100.0) {
                    observed.set(true);
                }
                let width = known.width.unwrap_or_else(|| match available.width {
                    AvailableSpace::Definite(v) => v.min(200.0),
                    _ => 200.0,
                });
                Size { width, height: if width < 200.0 { 20.0 } else { 10.0 } }
            },
        };
        let output = LayoutPartialTree::compute_child_layout(
            &mut view,
            root,
            LayoutInput {
                run_mode: RunMode::PerformLayout,
                rive_undefined_available: Size { width: Some(100.0), height: None },
                ..LayoutInput::HIDDEN
            },
        );
        assert!(observed.get());
        assert_eq!(output.size, Size { width: 100.0, height: 20.0 });
    }

    #[test]
    fn rive_wrapping_nonstretch_content_remeasures_hug_container_cross_axis() {
        use crate::style::{AlignContent, FlexDirection, FlexWrap};
        for row in [false, true] {
            let mut tree = TaffyTree::<()>::new();
            tree.disable_rounding();
            let mut hugs = vec![];
            for cross in [167.0, 279.0, 170.0] {
                let a = tree
                    .new_leaf(Style {
                        size: if row {
                            Size { width: length(10.0), height: length(100.0) }
                        } else {
                            Size { width: length(100.0), height: length(10.0) }
                        },
                        ..Style::default()
                    })
                    .unwrap();
                let b = tree
                    .new_leaf(Style {
                        size: if row {
                            Size { width: length(10.0), height: length(cross - 100.0) }
                        } else {
                            Size { width: length(cross - 100.0), height: length(10.0) }
                        },
                        ..Style::default()
                    })
                    .unwrap();
                hugs.push(
                    tree.new_with_children(
                        Style {
                            flex_direction: if row { FlexDirection::Column } else { FlexDirection::Row },
                            flex_grow: 1.0,
                            flex_shrink: 0.0,
                            min_size: Size { width: length(0.0), height: length(0.0) },
                            ..Style::default()
                        },
                        &[a, b],
                    )
                    .unwrap(),
                );
            }
            let root = tree
                .new_with_children(
                    Style {
                        size: Size { width: length(500.0), height: length(500.0) },
                        flex_direction: if row { FlexDirection::Row } else { FlexDirection::Column },
                        flex_wrap: FlexWrap::Wrap,
                        align_content: Some(AlignContent::FLEX_START),
                        ..Style::default()
                    },
                    &hugs,
                )
                .unwrap();
            tree.compute_layout_with_measure_and_probe(
                root,
                Size::MAX_CONTENT,
                |_, _, _| RiveMeasureMetadata::default(),
                |known, _, _, _, _, _| known.unwrap_or(Size::ZERO),
            )
            .unwrap();
            // Pinned Yoga's final kStretch pass makes the wrapping cross axis
            // Undefined when align-content does not grow. Initial exact sizing
            // must not overwrite intrinsic widths with the parent's500. Step8
            // then remeasures all siblings at the shared measured maximum279.
            for hug in hugs {
                let size = tree.layout(hug).unwrap().size;
                assert_eq!(if row { size.height } else { size.width }, 279.0);
                assert!(((if row { size.width } else { size.height }) - 500.0 / 3.0).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn rive_completed_results_survive_callback_mutation_until_dirt_or_direction_changes() {
        let mut tree = TaffyTree::<()>::new();
        let node = tree.new_leaf_with_context(Style::default(), ()).unwrap();
        let calls = core::cell::Cell::new(0);
        let content = core::cell::Cell::new(10.0);
        let direction = core::cell::Cell::new(0);
        let mut run = |tree: &mut TaffyTree<()>| {
            tree.compute_layout_with_measure_and_probe(
                node,
                Size::MAX_CONTENT,
                |_, _, _| RiveMeasureMetadata { owner_direction: direction.get(), normalization: 0 },
                |_, _, _, _, _, _| {
                    calls.set(calls.get() + 1);
                    let width = content.get();
                    content.set(width + 10.0);
                    Size { width, height: 10.0 }
                },
            )
            .unwrap();
        };
        run(&mut tree);
        run(&mut tree);
        assert_eq!(calls.get(), 1);
        assert_eq!(tree.layout(node).unwrap().size.width, 10.0);
        tree.mark_dirty(node).unwrap();
        run(&mut tree);
        assert_eq!(calls.get(), 2);
        direction.set(1);
        run(&mut tree);
        direction.set(0);
        run(&mut tree);
        assert_eq!(calls.get(), 4);
    }

    #[test]
    fn new_should_allocate_default_capacity() {
        const DEFAULT_CAPACITY: usize = 16; // This is the capacity defined in the `impl Default`
        let taffy: TaffyTree<()> = TaffyTree::new();

        assert!(taffy.children.capacity() >= DEFAULT_CAPACITY);
        assert!(taffy.parents.capacity() >= DEFAULT_CAPACITY);
        assert!(taffy.nodes.capacity() >= DEFAULT_CAPACITY);
    }

    #[test]
    fn test_with_capacity() {
        const CAPACITY: usize = 8;
        let taffy: TaffyTree<()> = TaffyTree::with_capacity(CAPACITY);

        assert!(taffy.children.capacity() >= CAPACITY);
        assert!(taffy.parents.capacity() >= CAPACITY);
        assert!(taffy.nodes.capacity() >= CAPACITY);
    }

    #[test]
    fn test_new_leaf() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let res = taffy.new_leaf(Style::default());
        assert!(res.is_ok());
        let node = res.unwrap();

        // node should be in the taffy tree and have no children
        assert!(taffy.child_count(node) == 0);
    }

    #[test]
    fn new_leaf_with_context() {
        let mut taffy: TaffyTree<Size<f32>> = TaffyTree::new();

        let res = taffy.new_leaf_with_context(Style::default(), Size::ZERO);
        assert!(res.is_ok());
        let node = res.unwrap();

        // node should be in the taffy tree and have no children
        assert!(taffy.child_count(node) == 0);
    }

    /// Test that new_with_children works as expected
    #[test]
    fn test_new_with_children() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        // node should have two children
        assert_eq!(taffy.child_count(node), 2);
        assert_eq!(taffy.children(node).unwrap()[0], child0);
        assert_eq!(taffy.children(node).unwrap()[1], child1);
    }

    #[test]
    fn remove_node_should_remove() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let node = taffy.new_leaf(Style::default()).unwrap();

        let _ = taffy.remove(node).unwrap();
    }

    #[test]
    fn remove_node_should_detach_hierarchy() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        // Build a linear tree layout: <0> <- <1> <- <2>
        let node2 = taffy.new_leaf(Style::default()).unwrap();
        let node1 = taffy.new_with_children(Style::default(), &[node2]).unwrap();
        let node0 = taffy.new_with_children(Style::default(), &[node1]).unwrap();

        // Both node0 and node1 should have 1 child nodes
        assert_eq!(taffy.children(node0).unwrap().as_slice(), &[node1]);
        assert_eq!(taffy.children(node1).unwrap().as_slice(), &[node2]);

        // Disconnect the tree: <0> <2>
        let _ = taffy.remove(node1).unwrap();

        // Both remaining nodes should have no child nodes
        assert!(taffy.children(node0).unwrap().is_empty());
        assert!(taffy.children(node2).unwrap().is_empty());
    }

    #[test]
    fn remove_last_node() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let parent = taffy.new_leaf(Style::default()).unwrap();
        let child = taffy.new_leaf(Style::default()).unwrap();
        taffy.add_child(parent, child).unwrap();

        taffy.remove(child).unwrap();
        taffy.remove(parent).unwrap();
    }

    #[test]
    fn set_measure() {
        let mut taffy: TaffyTree<Size<f32>> = TaffyTree::new();
        let node = taffy.new_leaf_with_context(Style::default(), Size { width: 200.0, height: 200.0 }).unwrap();
        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, size_measure_function).unwrap();
        assert_eq!(taffy.layout(node).unwrap().size.width, 200.0);

        taffy.set_node_context(node, Some(Size { width: 100.0, height: 100.0 })).unwrap();
        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, size_measure_function).unwrap();
        assert_eq!(taffy.layout(node).unwrap().size.width, 100.0);
    }

    #[test]
    fn set_measure_of_previously_unmeasured_node() {
        let mut taffy: TaffyTree<Size<f32>> = TaffyTree::new();
        let node = taffy.new_leaf(Style::default()).unwrap();
        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, size_measure_function).unwrap();
        assert_eq!(taffy.layout(node).unwrap().size.width, 0.0);

        taffy.set_node_context(node, Some(Size { width: 100.0, height: 100.0 })).unwrap();
        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, size_measure_function).unwrap();
        assert_eq!(taffy.layout(node).unwrap().size.width, 100.0);
    }

    /// Test that adding `add_child()` works
    #[test]
    fn add_child() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let node = taffy.new_leaf(Style::default()).unwrap();
        assert_eq!(taffy.child_count(node), 0);

        let child0 = taffy.new_leaf(Style::default()).unwrap();
        taffy.add_child(node, child0).unwrap();
        assert_eq!(taffy.child_count(node), 1);

        let child1 = taffy.new_leaf(Style::default()).unwrap();
        taffy.add_child(node, child1).unwrap();
        assert_eq!(taffy.child_count(node), 2);
    }

    #[test]
    fn insert_child_at_index() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let child2 = taffy.new_leaf(Style::default()).unwrap();

        let node = taffy.new_leaf(Style::default()).unwrap();
        assert_eq!(taffy.child_count(node), 0);

        taffy.insert_child_at_index(node, 0, child0).unwrap();
        assert_eq!(taffy.child_count(node), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child0);

        taffy.insert_child_at_index(node, 0, child1).unwrap();
        assert_eq!(taffy.child_count(node), 2);
        assert_eq!(taffy.children(node).unwrap()[0], child1);
        assert_eq!(taffy.children(node).unwrap()[1], child0);

        taffy.insert_child_at_index(node, 1, child2).unwrap();
        assert_eq!(taffy.child_count(node), 3);
        assert_eq!(taffy.children(node).unwrap()[0], child1);
        assert_eq!(taffy.children(node).unwrap()[1], child2);
        assert_eq!(taffy.children(node).unwrap()[2], child0);
    }

    #[test]
    fn set_children() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        assert_eq!(taffy.child_count(node), 2);
        assert_eq!(taffy.children(node).unwrap()[0], child0);
        assert_eq!(taffy.children(node).unwrap()[1], child1);

        let child2 = taffy.new_leaf(Style::default()).unwrap();
        let child3 = taffy.new_leaf(Style::default()).unwrap();
        taffy.set_children(node, &[child2, child3]).unwrap();

        assert_eq!(taffy.child_count(node), 2);
        assert_eq!(taffy.children(node).unwrap()[0], child2);
        assert_eq!(taffy.children(node).unwrap()[1], child3);
    }

    /// Test that removing a child works
    #[test]
    fn remove_child() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        assert_eq!(taffy.child_count(node), 2);

        taffy.remove_child(node, child0).unwrap();
        assert_eq!(taffy.child_count(node), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child1);

        taffy.remove_child(node, child1).unwrap();
        assert_eq!(taffy.child_count(node), 0);
    }

    #[test]
    fn remove_child_at_index() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        assert_eq!(taffy.child_count(node), 2);

        taffy.remove_child_at_index(node, 0).unwrap();
        assert_eq!(taffy.child_count(node), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child1);

        taffy.remove_child_at_index(node, 0).unwrap();
        assert_eq!(taffy.child_count(node), 0);
    }

    #[test]
    fn remove_children_range() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let child2 = taffy.new_leaf(Style::default()).unwrap();
        let child3 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1, child2, child3]).unwrap();

        assert_eq!(taffy.child_count(node), 4);

        taffy.remove_children_range(node, 1..=2).unwrap();
        assert_eq!(taffy.child_count(node), 2);
        assert_eq!(taffy.children(node).unwrap(), [child0, child3]);
        for child in [child0, child3] {
            assert_eq!(taffy.parent(child), Some(node));
        }
        for child in [child1, child2] {
            assert_eq!(taffy.parent(child), None);
        }
    }

    // Related to: https://github.com/DioxusLabs/taffy/issues/510
    #[test]
    fn remove_child_updates_parents() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let parent = taffy.new_leaf(Style::default()).unwrap();
        let child = taffy.new_leaf(Style::default()).unwrap();

        taffy.add_child(parent, child).unwrap();

        taffy.remove(parent).unwrap();

        // Once the parent is removed this shouldn't panic.
        assert!(taffy.set_children(child, &[]).is_ok());
    }

    #[test]
    fn replace_child_at_index() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();

        let node = taffy.new_with_children(Style::default(), &[child0]).unwrap();
        assert_eq!(taffy.child_count(node), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child0);

        taffy.replace_child_at_index(node, 0, child1).unwrap();
        assert_eq!(taffy.child_count(node), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child1);
    }
    #[test]
    fn test_child_at_index() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let child2 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1, child2]).unwrap();

        assert!(if let Ok(result) = taffy.child_at_index(node, 0) { result == child0 } else { false });
        assert!(if let Ok(result) = taffy.child_at_index(node, 1) { result == child1 } else { false });
        assert!(if let Ok(result) = taffy.child_at_index(node, 2) { result == child2 } else { false });
    }
    #[test]
    fn test_child_count() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        assert!(taffy.child_count(node) == 2);
        assert!(taffy.child_count(child0) == 0);
        assert!(taffy.child_count(child1) == 0);
    }

    #[allow(clippy::vec_init_then_push)]
    #[test]
    fn test_children() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        let mut children = sys::Vec::new();
        children.push(child0);
        children.push(child1);

        let children_result = taffy.children(node).unwrap();
        assert_eq!(children_result, children);

        assert!(taffy.children(child0).unwrap().is_empty());
    }
    #[test]
    fn test_set_style() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let node = taffy.new_leaf(Style::default()).unwrap();
        assert_eq!(taffy.style(node).unwrap().display, Display::Flex);

        taffy.set_style(node, Style { display: Display::None, ..Style::default() }).unwrap();
        assert_eq!(taffy.style(node).unwrap().display, Display::None);
    }
    #[test]
    fn test_style() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let style = Style { display: Display::None, flex_direction: FlexDirection::RowReverse, ..Default::default() };

        let node = taffy.new_leaf(style.clone()).unwrap();

        let res = taffy.style(node);
        assert!(res.is_ok());
        assert!(res.unwrap() == &style);
    }
    #[test]
    fn test_layout() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let node = taffy.new_leaf(Style::default()).unwrap();

        // TODO: Improve this test?
        let res = taffy.layout(node);
        assert!(res.is_ok());
    }

    #[test]
    fn test_mark_dirty() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        taffy.compute_layout(node, Size::MAX_CONTENT).unwrap();

        assert_eq!(taffy.dirty(child0), Ok(false));
        assert_eq!(taffy.dirty(child1), Ok(false));
        assert_eq!(taffy.dirty(node), Ok(false));

        taffy.mark_dirty(node).unwrap();
        assert_eq!(taffy.dirty(child0), Ok(false));
        assert_eq!(taffy.dirty(child1), Ok(false));
        assert_eq!(taffy.dirty(node), Ok(true));

        taffy.compute_layout(node, Size::MAX_CONTENT).unwrap();
        taffy.mark_dirty(child0).unwrap();
        assert_eq!(taffy.dirty(child0), Ok(true));
        assert_eq!(taffy.dirty(child1), Ok(false));
        assert_eq!(taffy.dirty(node), Ok(true));
    }

    #[test]
    fn compute_layout_should_produce_valid_result() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let node_result = taffy.new_leaf(Style {
            size: Size { width: Dimension::from_length(10f32), height: Dimension::from_length(10f32) },
            ..Default::default()
        });
        assert!(node_result.is_ok());
        let node = node_result.unwrap();
        let layout_result = taffy.compute_layout(
            node,
            Size { width: AvailableSpace::Definite(100.), height: AvailableSpace::Definite(100.) },
        );
        assert!(layout_result.is_ok());
    }

    #[test]
    fn make_sure_layout_location_is_top_left() {
        use crate::prelude::*;

        let mut taffy: TaffyTree<()> = TaffyTree::new();

        let node = taffy
            .new_leaf(Style {
                size: Size { width: Dimension::from_percent(1f32), height: Dimension::from_percent(1f32) },
                ..Default::default()
            })
            .unwrap();

        let root = taffy
            .new_with_children(
                Style {
                    size: Size { width: Dimension::from_length(100f32), height: Dimension::from_length(100f32) },
                    padding: Rect {
                        left: length(10f32),
                        right: length(20f32),
                        top: length(30f32),
                        bottom: length(40f32),
                    },
                    ..Default::default()
                },
                &[node],
            )
            .unwrap();

        taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();

        // If Layout::location represents top-left coord, 'node' location
        // must be (due applied 'root' padding): {x: 10, y: 30}.
        //
        // It's important, since result will be different for each other
        // coordinate space:
        // - bottom-left:  {x: 10, y: 40}
        // - top-right:    {x: 20, y: 30}
        // - bottom-right: {x: 20, y: 40}
        let layout = taffy.layout(node).unwrap();
        assert_eq!(layout.location.x, 10f32);
        assert_eq!(layout.location.y, 30f32);
    }

    #[test]
    fn set_children_reparents() {
        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let child = taffy.new_leaf(Style::default()).unwrap();
        let old_parent = taffy.new_with_children(Style::default(), &[child]).unwrap();

        let new_parent = taffy.new_leaf(Style::default()).unwrap();
        taffy.set_children(new_parent, &[child]).unwrap();

        assert!(taffy.children(old_parent).unwrap().is_empty());
    }
}
