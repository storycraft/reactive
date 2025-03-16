mod iter;

use iter::Iter;
use taffy::{
    AvailableSpace, CacheTree, Display, Layout, NodeId, Size, Style, compute_block_layout,
    compute_cached_layout, compute_flexbox_layout, compute_grid_layout, compute_hidden_layout,
    compute_leaf_layout,
};

use crate::ElementId;

use super::UiTree;

impl taffy::TraverseTree for UiTree {}

impl taffy::TraversePartialTree for UiTree {
    type ChildIter<'a> = Iter<'a>;

    fn child_ids(&self, id: NodeId) -> Self::ChildIter<'_> {
        Iter {
            tree: self.relations[ElementId::from_taffy_id(id)].children.iter(),
        }
    }

    fn child_count(&self, id: NodeId) -> usize {
        self.relations[ElementId::from_taffy_id(id)].children.len()
    }

    fn get_child_id(&self, id: NodeId, index: usize) -> NodeId {
        self.relations[ElementId::from_taffy_id(id)].children[index].to_taffy_id()
    }
}

impl taffy::LayoutPartialTree for UiTree {
    type CoreContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_core_container_style(&self, id: NodeId) -> Self::CoreContainerStyle<'_> {
        &self.elements[ElementId::from_taffy_id(id)].node.style
    }

    fn set_unrounded_layout(&mut self, id: NodeId, layout: &Layout) {
        self.elements[ElementId::from_taffy_id(id)]
            .as_mut()
            .node_mut()
            .layout = *layout;
    }

    fn compute_child_layout(
        &mut self,
        node_id: NodeId,
        inputs: taffy::tree::LayoutInput,
    ) -> taffy::tree::LayoutOutput {
        compute_cached_layout(self, node_id, inputs, |tree, id, inputs| {
            let element = &tree.elements[ElementId::from_taffy_id(id)];
            let node = &element.node;
            let display_mode = node.style.display;
            let has_children = !tree.relations[ElementId::from_taffy_id(id)]
                .children
                .is_empty();

            match (display_mode, has_children) {
                (Display::None, _) => compute_hidden_layout(tree, id),
                (Display::Block, true) => compute_block_layout(tree, id, inputs),
                (Display::Flex, true) => compute_flexbox_layout(tree, id, inputs),
                (Display::Grid, true) => compute_grid_layout(tree, id, inputs),
                (_, false) => {
                    compute_leaf_layout(inputs, &node.style, |known_dimensions, available_space| {
                        element.measure(known_dimensions, available_space)
                    })
                }
            }
        })
    }
}

impl CacheTree for UiTree {
    fn cache_get(
        &self,
        id: NodeId,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.elements[ElementId::from_taffy_id(id)].node.cache.get(
            known_dimensions,
            available_space,
            run_mode,
        )
    }

    fn cache_store(
        &mut self,
        id: NodeId,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.elements[ElementId::from_taffy_id(id)]
            .as_mut()
            .node_mut()
            .cache
            .store(known_dimensions, available_space, run_mode, layout_output)
    }

    fn cache_clear(&mut self, id: NodeId) {
        self.elements[ElementId::from_taffy_id(id)]
            .as_mut()
            .node_mut()
            .cache
            .clear();
    }
}

impl taffy::LayoutFlexboxContainer for UiTree {
    type FlexboxContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type FlexboxItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_flexbox_container_style(&self, id: NodeId) -> Self::FlexboxContainerStyle<'_> {
        &self.elements[ElementId::from_taffy_id(id)].node.style
    }

    fn get_flexbox_child_style(&self, child_node_id: NodeId) -> Self::FlexboxItemStyle<'_> {
        self.get_flexbox_container_style(child_node_id)
    }
}

impl taffy::LayoutGridContainer for UiTree {
    type GridContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type GridItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_grid_container_style(&self, id: NodeId) -> Self::GridContainerStyle<'_> {
        &self.elements[ElementId::from_taffy_id(id)].node.style
    }

    fn get_grid_child_style(&self, child_node_id: NodeId) -> Self::GridItemStyle<'_> {
        self.get_grid_container_style(child_node_id)
    }
}

impl taffy::LayoutBlockContainer for UiTree {
    type BlockContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type BlockItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_block_container_style(&self, id: NodeId) -> Self::BlockContainerStyle<'_> {
        &self.elements[ElementId::from_taffy_id(id)].node.style
    }

    fn get_block_child_style(&self, child_node_id: NodeId) -> Self::BlockItemStyle<'_> {
        self.get_block_container_style(child_node_id)
    }
}
